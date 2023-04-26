use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint64;

use crate::ContractError;

#[cw_serde]
pub struct Epoch {
    pub duration: Uint64,
    pub start_time: Uint64,
    pub offset: Uint64,
}

impl Epoch {
    pub fn new(duration: Uint64, start_time: Uint64) -> Self {
        let offset = start_time % duration;

        Epoch {
            duration,
            start_time,
            offset,
        }
    }

    pub fn current_epoch(&self, seconds: Uint64) -> Uint64 {
        if seconds < self.start_time {
            return Uint64::from(0u64);
        }
        let adjusted_seconds = seconds - self.start_time + self.offset;
        adjusted_seconds / self.duration
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.duration == Uint64::zero() {
            return Err(ContractError::EpochDurationCannotBeZero {});
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ContractError;

    use super::Epoch;
    use cosmwasm_std::Uint64;

    #[test]
    fn test_epoch_new() {
        let epoch = Epoch::new(Uint64::from(100u64), Uint64::from(50u64));
        epoch.validate().unwrap();
        assert_eq!(epoch.duration, Uint64::from(100u64));
        assert_eq!(epoch.start_time, Uint64::from(50u64));
        assert_eq!(epoch.offset, Uint64::from(50u64));
    }

    #[test]
    fn test_current_epoch_before_start_time() {
        let epoch = Epoch::new(Uint64::from(100u64), Uint64::from(50u64));
        epoch.validate().unwrap();
        let current = epoch.current_epoch(Uint64::from(30u64));
        assert_eq!(current, Uint64::from(0u64));
    }

    #[test]
    fn test_current_epoch_at_start_time() {
        let epoch = Epoch::new(Uint64::from(100u64), Uint64::from(50u64));
        epoch.validate().unwrap();
        let current = epoch.current_epoch(Uint64::from(50u64));
        assert_eq!(current, Uint64::from(0u64));
    }

    #[test]
    fn test_current_epoch_in_first_epoch() {
        let epoch = Epoch::new(Uint64::from(100u64), Uint64::from(50u64));
        epoch.validate().unwrap();
        let current = epoch.current_epoch(Uint64::from(80u64));
        assert_eq!(current, Uint64::from(0u64));
    }

    #[test]
    fn test_current_epoch_at_second_epoch_start() {
        let epoch = Epoch::new(Uint64::from(100u64), Uint64::from(50u64));
        epoch.validate().unwrap();
        let current = epoch.current_epoch(Uint64::from(150u64));
        assert_eq!(current, Uint64::from(1u64));
    }

    #[test]
    fn test_current_epoch_in_second_epoch() {
        let epoch = Epoch::new(Uint64::from(100u64), Uint64::from(50u64));
        epoch.validate().unwrap();
        let current = epoch.current_epoch(Uint64::from(170u64));
        assert_eq!(current, Uint64::from(1u64));
    }

    #[test]
    fn test_current_epoch_in_larger_epoch() {
        let epoch = Epoch::new(Uint64::from(100u64), Uint64::from(50u64));
        epoch.validate().unwrap();
        let current = epoch.current_epoch(Uint64::from(1050u64));
        assert_eq!(current, Uint64::from(10u64));
    }

    #[test]
    #[should_panic]
    fn test_current_epoch_with_zero_duration() {
        let epoch = Epoch {
            duration: Uint64::zero(),
            start_time: Uint64::from(50u64),
            offset: Uint64::from(50u64),
        };
        epoch.current_epoch(Uint64::from(170u64));
    }

    #[test]
    #[should_panic]
    fn test_new_with_zero_duration() {
        Epoch::new(Uint64::zero(), Uint64::from(50u64));
    }

    #[test]
    fn test_validate_with_zero_duration() {
        let epoch = Epoch {
            duration: Uint64::zero(),
            start_time: Uint64::from(50u64),
            offset: Uint64::from(50u64),
        };
        assert_eq!(
            epoch.validate().unwrap_err(),
            ContractError::EpochDurationCannotBeZero {}
        );
    }
}
