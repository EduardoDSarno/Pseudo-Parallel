#[derive(Debug, Clone)]

/* This struct is responsible for holding dumb data, basically for each price level entry in the book
    we just need to know how many subscribers that are and the triggering price */
pub struct PriceLevelEntry 
{
    pub trigger_price: f64,
    subscriber_count: usize,
}

impl PriceLevelEntry 
{
    pub fn new(trigger_price: f64) -> Self {
        PriceLevelEntry {
            trigger_price,
            subscriber_count: 1,
        }
    }

    pub fn add_subscriber(&mut self) {
        self.subscriber_count += 1;
    }

    pub fn remove_subscriber(&mut self) -> bool {
        self.subscriber_count -= 1;
        self.subscriber_count == 0
    }

    pub fn subscriber_count(&self) -> usize {
        self.subscriber_count
    }
}
