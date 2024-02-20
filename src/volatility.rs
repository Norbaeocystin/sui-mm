use statistical::standard_deviation;
use statistical::mean;

#[derive(Debug, Clone)]
pub struct Volatility {
    pub prices: Vec<f64>,
    pub length: usize,
}

impl Volatility {
    // the recent value is first value
    pub fn insert(&mut self,  value: f64){
            if self.prices.len() > 0 {
                self.prices.insert(0, value);
            } else {
                self.prices.push(value);
            }
            if self.prices.len() > self.length {
                self.prices.truncate(self.length);
            }
    }

    pub fn volatility(self) -> Option<f64> {
        if self.prices.len() == self.length {
            let mean_val = mean(&self.prices);
            return Some((standard_deviation((&self.prices), None)/mean_val) * (self.length as f64).sqrt());
        } else {
            return None;
        }
    }

    pub fn is_ok(self) -> bool {
        return self.prices.len() == self.length;
    }
}
