# Response to plan
## Things that I have added you should be aware of
- Updated .clear_all() to clear() and made it use the Default impl instead of manually unsetting everything.
- Added a .retain_config() method to set everything but config-related fields to default
- The above allows us in .is_empty() to do an easy impl to get a version that is defaulted (other than config) and then to compare to that to see if the CardFilter is empty
