#!/bin/bash
# set -e # Disabled to allow non-blocking execution

ENDPOINT_URL="${AWS_ENDPOINT_URL:-https://localhost.localstack.cloud:4566}"

# Define awslocal wrapper to enforce endpoint and disable pager for non-interactive mode
function awslocal() {
    aws --endpoint-url "$ENDPOINT_URL" --no-cli-pager "$@"
}

REGION="us-east-1"
KEY_PAIR_NAME="MyKeyPair"

# Export dummy credentials
export AWS_ACCESS_KEY_ID="test"
export AWS_SECRET_ACCESS_KEY="test"
export AWS_DEFAULT_REGION="${REGION}"

echo "Seeding LocalStack resources..."

# Helper function to check if bucket exists
bucket_exists() {
    awslocal s3api head-bucket --bucket "$1" 2>/dev/null
}

# Create buckets
echo "Creating buckets..."
for bucket in test-bucket-1 test-bucket-2 empty-bucket versioned-bucket; do
    if bucket_exists "$bucket"; then
        echo "Bucket $bucket already exists, skipping creation."
    else
        echo "Creating bucket $bucket..."
        awslocal s3 mb "s3://$bucket"
    fi
done

# Enable versioning on versioned-bucket
echo "Enabling versioning on versioned-bucket..."
awslocal s3api put-bucket-versioning --bucket versioned-bucket --versioning-configuration Status=Enabled

# Create some dummy files
echo "Creating dummy files..."
echo "Hello World" > hello.txt
echo "Another file" > data.json
echo "Log entry" > app.log

# Upload files (always upload to ensure content exists)
echo "Uploading objects..."
awslocal s3 cp hello.txt s3://test-bucket-1/
awslocal s3 cp data.json s3://test-bucket-1/
awslocal s3 cp app.log s3://test-bucket-2/logs/app.log

# Upload multiple versions to versioned bucket
echo "Uploading versions..."
echo "Version 1" > versioned.txt
awslocal s3 cp versioned.txt s3://versioned-bucket/
echo "Version 2" > versioned.txt
awslocal s3 cp versioned.txt s3://versioned-bucket/
echo "Version 3" > versioned.txt
awslocal s3 cp versioned.txt s3://versioned-bucket/

# Clean up local files
rm hello.txt data.json app.log versioned.txt

# Create Backup Vault
VAULT_NAME="TestVault"
# moto doesn't support describe-backup-vault via CLI well, using list to check
if awslocal backup list-backup-vaults | grep -q "$VAULT_NAME"; then
    echo "Backup vault $VAULT_NAME already exists."
else
    echo "Creating backup vault $VAULT_NAME..."
    awslocal backup create-backup-vault --backup-vault-name "$VAULT_NAME"
fi

# Create Backup Plan
PLAN_NAME="TestBackupPlan"
if awslocal backup list-backup-plans | grep -q "$PLAN_NAME"; then
    echo "Backup plan $PLAN_NAME already exists."
else
    echo "Creating backup plan $PLAN_NAME..."
    cat <<EOF > backup-plan.json
{
    "BackupPlanName": "$PLAN_NAME",
    "Rules": [
        {
            "RuleName": "DailyBackups",
            "TargetBackupVaultName": "$VAULT_NAME",
            "ScheduleExpression": "cron(0 12 * * ? *)",
            "StartWindowMinutes": 60,
            "CompletionWindowMinutes": 180,
            "Lifecycle": {
                "DeleteAfterDays": 30
            }
        }
    ]
}
EOF
    awslocal backup create-backup-plan --backup-plan file://backup-plan.json
    rm backup-plan.json

    # Create Backup Selection for EC2 instances with Backup:True tag
    SELECTION_NAME="EC2BackupSelection"
    echo "Creating backup selection $SELECTION_NAME..."
    PLAN_ID=$(awslocal backup list-backup-plans --query "BackupPlansList[?BackupPlanName=='$PLAN_NAME'].BackupPlanId" --output text)
    cat <<EOF > backup-selection.json
{
    "SelectionName": "$SELECTION_NAME",
    "IamRoleArn": "arn:aws:iam::000000000000:role/service-role/AWSBackupDefaultServiceRole",
    "Resources": [],
    "ListOfTags": [
        {
            "ConditionType": "STRINGEQUALS",
            "ConditionKey": "Backup",
            "ConditionValue": "True"
        }
    ],
    "Conditions": {
        "StringEquals": [
            {
                "ConditionKey": "aws:ResourceType",
                "ConditionValue": "EC2"
            }
        ]
    }
}
EOF
    awslocal backup create-backup-selection --backup-plan-id "$PLAN_ID" --backup-selection file://backup-selection.json
    rm backup-selection.json
fi

# EC2 Key Pair
echo "Checking EC2 Key Pair..."
if awslocal ec2 describe-key-pairs --key-names "$KEY_PAIR_NAME" 2>/dev/null; then
    echo "Key pair $KEY_PAIR_NAME already exists."
else
    echo "Key pair $KEY_PAIR_NAME not found."
    if [ -f "$HOME/.ssh/id_rsa.pub" ]; then
        echo "Found SSH public key at $HOME/.ssh/id_rsa.pub. Importing..."
        awslocal ec2 import-key-pair --key-name "$KEY_PAIR_NAME" --public-key-material "fileb://$HOME/.ssh/id_rsa.pub"
    elif [ -f "$HOME/.ssh/id_ed25519.pub" ]; then
        echo "Found SSH public key at $HOME/.ssh/id_ed25519.pub. Importing..."
        awslocal ec2 import-key-pair --key-name "$KEY_PAIR_NAME" --public-key-material "fileb://$HOME/.ssh/id_ed25519.pub"
    else
        echo "No SSH public key found. Creating a new one..."
        awslocal ec2 create-key-pair --key-name "$KEY_PAIR_NAME"
    fi
fi

# Create EC2 Instance
INSTANCE_TAG="SeededInstance"
echo "Checking for seeded EC2 instance..."
# Check if any instance with the tag exists and is running/pending
EXISTING_INSTANCES=$(awslocal ec2 describe-instances --filters "Name=tag:Name,Values=$INSTANCE_TAG" "Name=instance-state-name,Values=pending,running" --query "Reservations[].Instances[].InstanceId" --output text)

if [ -n "$EXISTING_INSTANCES" ]; then
    echo "EC2 instance with tag Name=$INSTANCE_TAG already exists: $EXISTING_INSTANCES"
else
    echo "Creating EC2 instance..."
    awslocal ec2 run-instances \
        --image-id ami-df5de72bdb3b \
        --count 1 \
        --instance-type t2.micro \
        --key-name "$KEY_PAIR_NAME" \
        --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=$INSTANCE_TAG}, {Key=Backup,Value=True}]"
fi

# Create ECS Cluster
CLUSTER_NAME="TestCluster"
echo "Checking ECS Cluster..."
if awslocal ecs describe-clusters --clusters "$CLUSTER_NAME" | grep -q "ACTIVE"; then
    echo "ECS cluster $CLUSTER_NAME already exists."
else
    echo "Creating ECS cluster $CLUSTER_NAME..."
    awslocal ecs create-cluster --cluster-name "$CLUSTER_NAME"
fi

# Register ECS Task Definition
TASK_FAMILY="web-app"
echo "Checking ECS Task Definition..."
if awslocal ecs list-task-definitions --family-prefix "$TASK_FAMILY" | grep -q "$TASK_FAMILY"; then
    echo "Task definition $TASK_FAMILY already exists."
else
    echo "Registering task definition $TASK_FAMILY..."
    awslocal ecs register-task-definition \
        --family "$TASK_FAMILY" \
        --network-mode bridge \
        --container-definitions '[
            {
                "name": "web-container",
                "image": "nginx:latest",
                "memory": 128,
                "cpu": 100,
                "portMappings": [
                    {
                        "containerPort": 80,
                        "hostPort": 8080,
                        "protocol": "tcp"
                    }
                ]
            }
        ]'
fi

# Create ECS Service
SERVICE_NAME="WebService"
echo "Checking ECS Service..."
if awslocal ecs describe-services --cluster "$CLUSTER_NAME" --services "$SERVICE_NAME" | grep -q "ACTIVE"; then
    echo "ECS service $SERVICE_NAME already exists."
else
    echo "Creating ECS service $SERVICE_NAME..."
    awslocal ecs create-service \
        --cluster "$CLUSTER_NAME" \
        --service-name "$SERVICE_NAME" \
        --task-definition "$TASK_FAMILY" \
        --desired-count 1
fi

# --- Lambda Seeding ---
LAMBDA_NAME="HelloFunction"
echo "Checking Lambda Function..."
if awslocal lambda get-function --function-name "$LAMBDA_NAME" 2>/dev/null; then
    echo "Lambda function $LAMBDA_NAME already exists."
else
    echo "Creating Lambda function $LAMBDA_NAME..."
    echo "def lambda_handler(event, context): return 'Hello from LocalStack Lambda'" > lambda_function.py
    zip function.zip lambda_function.py
    awslocal lambda create-function \
        --function-name "$LAMBDA_NAME" \
        --zip-file fileb://function.zip \
        --handler lambda_function.lambda_handler \
        --runtime python3.8 \
        --role arn:aws:iam::000000000000:role/lambda-role \
        --tags "Environment=Dev,Project=TUI"
    rm lambda_function.py function.zip
fi

# --- RDS Seeding ---
DB_ID="test-db"
echo "Checking RDS Instance..."
if awslocal rds describe-db-instances --db-instance-identifier "$DB_ID" 2>/dev/null; then
    echo "RDS instance $DB_ID already exists."
else
    echo "Creating RDS instance $DB_ID..."
    awslocal rds create-db-instance \
        --db-instance-identifier "$DB_ID" \
        --db-instance-class db.t2.micro \
        --engine postgres \
        --allocated-storage 20 \
        --master-username postgres \
        --master-user-password postgres
fi

# --- DynamoDB Seeding ---
TABLE_NAME="TestTable"
echo "Checking DynamoDB Table..."
if awslocal dynamodb describe-table --table-name "$TABLE_NAME" 2>/dev/null; then
    echo "DynamoDB table $TABLE_NAME already exists."
else
    echo "Creating DynamoDB table $TABLE_NAME..."
    awslocal dynamodb create-table \
        --table-name "$TABLE_NAME" \
        --attribute-definitions AttributeName=PK,AttributeType=S AttributeName=SK,AttributeType=S \
        --key-schema AttributeName=PK,KeyType=HASH AttributeName=SK,KeyType=RANGE \
        --provisioned-throughput ReadCapacityUnits=5,WriteCapacityUnits=5 \
        --tags Key=Environment,Value=Test

    echo "Putting items into $TABLE_NAME..."
    awslocal dynamodb put-item --table-name "$TABLE_NAME" --item '{"PK": {"S": "USER#1"}, "SK": {"S": "PROFILE"}, "Name": {"S": "Alice"}}'
    awslocal dynamodb put-item --table-name "$TABLE_NAME" --item '{"PK": {"S": "USER#2"}, "SK": {"S": "PROFILE"}, "Name": {"S": "Bob"}}'
fi

# --- IAM Seeding ---
POLICY_NAME="TestPolicy"
echo "Checking IAM Policy..."
POLICY_ARN=$(awslocal iam list-policies --query "Policies[?PolicyName=='$POLICY_NAME'].Arn" --output text)
if [ -n "$POLICY_ARN" ] && [ "$POLICY_ARN" != "None" ]; then
    echo "IAM policy $POLICY_NAME already exists."
else
    echo "Creating IAM policy $POLICY_NAME..."
    cat <<EOF > policy.json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": "*",
      "Resource": "*"
    }
  ]
}
EOF
    POLICY_ARN=$(awslocal iam create-policy --policy-name "$POLICY_NAME" --policy-document file://policy.json --query 'Policy.Arn' --output text)
    rm policy.json
fi

USER_NAME="TestUser"
echo "Checking IAM User..."
if awslocal iam get-user --user-name "$USER_NAME" 2>/dev/null; then
    echo "IAM user $USER_NAME already exists."
else
    echo "Creating IAM user $USER_NAME..."
    awslocal iam create-user --user-name "$USER_NAME"

    echo "Creating Access Key for $USER_NAME..."
    awslocal iam create-access-key --user-name "$USER_NAME"

    echo "Attaching managed policy to $USER_NAME..."
    awslocal iam attach-user-policy --user-name "$USER_NAME" --policy-arn "$POLICY_ARN"

    echo "Putting inline policy to $USER_NAME..."
    cat <<EOF > inline_policy.json
{
  "Version": "2012-10-17",
  "Statement": [
    {
        "Effect": "Allow",
        "Action": "s3:ListBucket",
        "Resource": "arn:aws:s3:::example_bucket"
    }
  ]
}
EOF
    awslocal iam put-user-policy --user-name "$USER_NAME" --policy-name "UserInlinePolicy" --policy-document file://inline_policy.json
    rm inline_policy.json
fi

ROLE_NAME="TestRole"
echo "Checking IAM Role..."
if awslocal iam get-role --role-name "$ROLE_NAME" 2>/dev/null; then
    echo "IAM role $ROLE_NAME already exists."
else
    echo "Creating IAM role $ROLE_NAME..."
    cat <<EOF > trust-policy.json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": { "Service": "ec2.amazonaws.com" },
      "Action": "sts:AssumeRole"
    }
  ]
}
EOF
    awslocal iam create-role --role-name "$ROLE_NAME" --assume-role-policy-document file://trust-policy.json
    rm trust-policy.json

    echo "Attaching managed policy to $ROLE_NAME..."
    awslocal iam attach-role-policy --role-name "$ROLE_NAME" --policy-arn "$POLICY_ARN"

    echo "Putting inline policy to $ROLE_NAME..."
        cat <<EOF > role_inline_policy.json
{
  "Version": "2012-10-17",
  "Statement": [
    {
        "Effect": "Allow",
        "Action": "dynamodb:*",
        "Resource": "*"
    }
  ]
}
EOF
    awslocal iam put-role-policy --role-name "$ROLE_NAME" --policy-name "RoleInlinePolicy" --policy-document file://role_inline_policy.json
    rm role_inline_policy.json
fi


# --- CloudTrail Seeding ---
TRAIL_NAME="TestTrail"
TRAIL_BUCKET="cloudtrail-logs-bucket"

echo "Checking CloudTrail Trail..."
if awslocal cloudtrail describe-trails --trail-name-list "$TRAIL_NAME" | grep -q "$TRAIL_NAME"; then
    echo "CloudTrail trail $TRAIL_NAME already exists."
else
    # Create bucket for CloudTrail logs
    if bucket_exists "$TRAIL_BUCKET"; then
        echo "Bucket $TRAIL_BUCKET already exists."
    else
        echo "Creating CloudTrail logs bucket $TRAIL_BUCKET..."
        awslocal s3 mb "s3://$TRAIL_BUCKET"
    fi

    # Set bucket policy for CloudTrail
    echo "Setting bucket policy for CloudTrail..."
    cat <<EOF > trail-bucket-policy.json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "AWSCloudTrailAclCheck",
            "Effect": "Allow",
            "Principal": {"Service": "cloudtrail.amazonaws.com"},
            "Action": "s3:GetBucketAcl",
            "Resource": "arn:aws:s3:::$TRAIL_BUCKET"
        },
        {
            "Sid": "AWSCloudTrailWrite",
            "Effect": "Allow",
            "Principal": {"Service": "cloudtrail.amazonaws.com"},
            "Action": "s3:PutObject",
            "Resource": "arn:aws:s3:::$TRAIL_BUCKET/*",
            "Condition": {"StringEquals": {"s3:x-amz-acl": "bucket-owner-full-control"}}
        }
    ]
}
EOF
    awslocal s3api put-bucket-policy --bucket "$TRAIL_BUCKET" --policy file://trail-bucket-policy.json
    rm trail-bucket-policy.json

    echo "Creating CloudTrail trail $TRAIL_NAME..."
    awslocal cloudtrail create-trail \
        --name "$TRAIL_NAME" \
        --s3-bucket-name "$TRAIL_BUCKET" \
        --is-multi-region-trail

    echo "Starting logging for trail $TRAIL_NAME..."
    awslocal cloudtrail start-logging --name "$TRAIL_NAME"
fi

# Create a second trail (stopped) for testing
TRAIL_NAME_2="InactiveTrail"
if awslocal cloudtrail describe-trails --trail-name-list "$TRAIL_NAME_2" | grep -q "$TRAIL_NAME_2"; then
    echo "CloudTrail trail $TRAIL_NAME_2 already exists."
else
    echo "Creating CloudTrail trail $TRAIL_NAME_2..."
    awslocal cloudtrail create-trail \
        --name "$TRAIL_NAME_2" \
        --s3-bucket-name "$TRAIL_BUCKET"
    # Don't start logging - leave it inactive for testing
fi

# --- Secrets Manager Seeding ---
SECRET_NAME="MyFirstSecret"
echo "Checking Secrets Manager Secret..."
if awslocal secretsmanager list-secrets | grep -q "$SECRET_NAME"; then
    echo "Secret $SECRET_NAME already exists."
else
    echo "Creating secret $SECRET_NAME..."
    awslocal secretsmanager create-secret \
        --name "$SECRET_NAME" \
        --description "This is a seeded secret" \
        --secret-string '{"username":"admin","password":"password123"}' \
        --tags Key=Environment,Value=Dev
fi

SECRET_NAME_2="ApiKey"
if awslocal secretsmanager list-secrets | grep -q "$SECRET_NAME_2"; then
    echo "Secret $SECRET_NAME_2 already exists."
else
    echo "Creating secret $SECRET_NAME_2..."
    awslocal secretsmanager create-secret \
        --name "$SECRET_NAME_2" \
        --description "API Key for external service" \
        --secret-string "ak_1234567890" \
        --tags Key=Environment,Value=Prod
fi

echo "Seeding complete!"
