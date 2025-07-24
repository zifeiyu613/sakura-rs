use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    // 以最小单位存储，例如人民币分、美分
    pub amount: i64,
    pub currency: Currency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    CNY,
    USD,
    EUR,
    GBP,
    JPY,
    // 其他货币...
}

impl Money {
    pub fn new(amount: i64, currency: Currency) -> Self {
        Self { amount, currency }
    }

    pub fn cny(amount: i64) -> Self {
        Self { amount, currency: Currency::CNY }
    }

    pub fn usd(amount: i64) -> Self {
        Self { amount, currency: Currency::USD }
    }

    // 简单货币操作
    pub fn add(&self, other: &Self) -> Result<Self, &'static str> {
        if self.currency != other.currency {
            return Err("Cannot add different currencies");
        }

        Ok(Self {
            amount: self.amount + other.amount,
            currency: self.currency,
        })
    }

    pub fn subtract(&self, other: &Self) -> Result<Self, &'static str> {
        if self.currency != other.currency {
            return Err("Cannot subtract different currencies");
        }

        Ok(Self {
            amount: self.amount - other.amount,
            currency: self.currency,
        })
    }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.currency {
            Currency::CNY => write!(f, "¥{:.2}", self.amount as f64 / 100.0),
            Currency::USD => write!(f, "${:.2}", self.amount as f64 / 100.0),
            Currency::EUR => write!(f, "€{:.2}", self.amount as f64 / 100.0),
            Currency::GBP => write!(f, "£{:.2}", self.amount as f64 / 100.0),
            Currency::JPY => write!(f, "¥{}", self.amount), // JPY没有小数点
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_creation() {
        let m1 = Money::cny(1000);
        assert_eq!(m1.amount, 1000);
        assert_eq!(m1.currency, Currency::CNY);

        let m2 = Money::usd(1000);
        assert_eq!(m2.amount, 1000);
        assert_eq!(m2.currency, Currency::USD);
    }

    #[test]
    fn test_money_add() {
        let m1 = Money::cny(1000);
        let m2 = Money::cny(500);
        let result = m1.add(&m2).unwrap();
        assert_eq!(result.amount, 1500);
        assert_eq!(result.currency, Currency::CNY);
    }

    #[test]
    fn test_money_subtract() {
        let m1 = Money::cny(1000);
        let m2 = Money::cny(300);
        let result = m1.subtract(&m2).unwrap();
        assert_eq!(result.amount, 700);
        assert_eq!(result.currency, Currency::CNY);
    }

    #[test]
    fn test_different_currency_operations() {
        let m1 = Money::cny(1000);
        let m2 = Money::usd(200);

        assert!(m1.add(&m2).is_err());
        assert!(m1.subtract(&m2).is_err());
    }

    #[test]
    fn test_display_format() {
        let m1 = Money::cny(1050);
        assert_eq!(format!("{}", m1), "¥10.50");

        let m2 = Money::usd(1999);
        assert_eq!(format!("{}", m2), "$19.99");
    }
}