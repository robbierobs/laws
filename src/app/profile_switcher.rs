//! Profile and Region Switcher State

/// State for the profile and region switcher modal
pub struct ProfileSwitcherState {
    pub available_profiles: Vec<String>,
    pub available_regions: Vec<String>,
    pub profile_switcher_index: usize,
    pub region_switcher_index: usize,
    pub pending_profile: Option<String>,
    pub pending_read_only: bool,
    pub profile_filter: String,
    pub region_filter: String,
    pub profile_filter_active: bool,
    pub region_filter_active: bool,
}

impl Default for ProfileSwitcherState {
    fn default() -> Self {
        Self {
            available_profiles: Vec::new(),
            available_regions: Vec::new(),
            profile_switcher_index: 0,
            region_switcher_index: 0,
            pending_profile: None,
            pending_read_only: false,
            profile_filter: String::new(),
            region_filter: String::new(),
            profile_filter_active: false,
            region_filter_active: false,
        }
    }
}

impl ProfileSwitcherState {
    pub fn new(current_profile: Option<&str>, current_region: &str, read_only: bool) -> Self {
        // Load available profiles from AWS config
        let available_profiles = crate::utils::aws_profiles::list_profiles();
        let available_regions: Vec<String> = crate::utils::aws_profiles::ALL_REGIONS
            .iter()
            .map(|s| s.to_string())
            .collect();

        // Find current profile/region index for pre-selection
        let profile_switcher_index = current_profile
            .and_then(|p| available_profiles.iter().position(|x| x == p))
            .unwrap_or(0);
        
        let region_switcher_index = available_regions
            .iter()
            .position(|r| r == current_region)
            .unwrap_or(0);

        Self {
            available_profiles,
            available_regions,
            profile_switcher_index,
            region_switcher_index,
            pending_profile: None,
            pending_read_only: read_only,
            profile_filter: String::new(),
            region_filter: String::new(),
            profile_filter_active: false,
            region_filter_active: false,
        }
    }

    /// Get filtered profile list based on current filter
    pub fn filtered_profiles(&self) -> Vec<&String> {
        if self.profile_filter.is_empty() {
            self.available_profiles.iter().collect()
        } else {
            let filter_lower = self.profile_filter.to_lowercase();
            self.available_profiles
                .iter()
                .filter(|p| p.to_lowercase().contains(&filter_lower))
                .collect()
        }
    }

    /// Get filtered region list based on current filter
    pub fn filtered_regions(&self) -> Vec<&String> {
        if self.region_filter.is_empty() {
            self.available_regions.iter().collect()
        } else {
            let filter_lower = self.region_filter.to_lowercase();
            self.available_regions
                .iter()
                .filter(|r| r.to_lowercase().contains(&filter_lower))
                .collect()
        }
    }

    pub fn reset_filters(&mut self) {
        self.profile_filter.clear();
        self.region_filter.clear();
        self.profile_filter_active = false;
        self.region_filter_active = false;
    }
    
    pub fn nav_down_profile(&mut self) {
        let len = self.filtered_profiles().len();
        if len > 0 {
            self.profile_switcher_index = (self.profile_switcher_index + 1) % len;
        }
    }

    pub fn nav_up_profile(&mut self) {
        let len = self.filtered_profiles().len();
        if len > 0 {
            self.profile_switcher_index = if self.profile_switcher_index == 0 {
                len - 1
            } else {
                self.profile_switcher_index - 1
            };
        }
    }

    pub fn nav_down_region(&mut self) {
        let len = self.filtered_regions().len();
        if len > 0 {
            self.region_switcher_index = (self.region_switcher_index + 1) % len;
        }
    }

    pub fn nav_up_region(&mut self) {
        let len = self.filtered_regions().len();
        if len > 0 {
            self.region_switcher_index = if self.region_switcher_index == 0 {
                len - 1
            } else {
                self.region_switcher_index - 1
            };
        }
    }
}
