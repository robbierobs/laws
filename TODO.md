# laws TODO

## Future Feature Considerations

### Cost Explorer Integration
**Status**: Deferred (API costs $0.01 per call)

AWS Cost Explorer (`aws-sdk-costexplorer`) would enable:
- MTD/monthly cost summaries and trends
- Cost breakdown by service, region, account, tags
- Forecasting and anomaly detection
- Drill-down from service → usage type → resource

**Reason for deferral**: Each API call has an associated cost. May revisit if there's demand and a smart caching/usage strategy can be implemented.

---

*Last updated: 2026-01-02*
