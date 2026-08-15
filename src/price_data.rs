use std::{
    collections::VecDeque,
    time::{Duration, SystemTime},
};

use hypersdk::Decimal;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PricePoint {
    price: Decimal,
    timestamp: SystemTime,
}

impl PricePoint {
    pub fn new(price: Decimal, timestamp: SystemTime) -> Self {
        Self { price, timestamp }
    }

    pub fn price(&self) -> Decimal {
        self.price
    }

    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
}

pub struct PriceWindow {
    /// Points ordered from oldest to newest.
    price_points: VecDeque<PricePoint>,
    /// Maximum age of a point relative to the newest point.
    duration: Duration,
    /// Hard memory bound for the number of retained points.
    max_points: usize,
}

impl PriceWindow {
    pub fn new(duration: Duration, max_points: usize) -> Self {
        assert!(max_points > 0, "max_points must be greater than zero");

        Self {
            price_points: VecDeque::new(),
            duration,
            max_points,
        }
    }

    pub fn push(&mut self, point: PricePoint) {
        // Keeping points chronological makes the front/back calculations valid.
        if self
            .newest()
            .is_some_and(|newest| point.timestamp() < newest.timestamp())
        {
            return;
        }

        let newest_timestamp = point.timestamp();
        self.price_points.push_back(point);

        // Remove points that fall outside the configured time window. A point
        // exactly `duration` old is still part of the window.
        while self.price_points.front().is_some_and(|oldest| {
            newest_timestamp
                .duration_since(oldest.timestamp())
                .is_ok_and(|age| age > self.duration)
        }) {
            self.price_points.pop_front();
        }

        // Protect memory even if the feed produces an unusually high number of
        // updates during the configured duration.
        while self.price_points.len() > self.max_points {
            self.price_points.pop_front();
        }
    }

    pub fn oldest(&self) -> Option<&PricePoint> {
        self.price_points.front()
    }

    pub fn newest(&self) -> Option<&PricePoint> {
        self.price_points.back()
    }

    /// Calculate the percentage change of price in the VecDeque
    pub fn percentage_change(&self) -> Option<Decimal> {
        if self.price_points.len() < 2 {
            return None;
        }

        let oldest_price = self.oldest()?.price();
        let newest_price = self.newest()?.price();

        if oldest_price == Decimal::ZERO {
            return None;
        }

        // converts to Decimal and use 100 to convert to percent
        Some((newest_price - oldest_price) / oldest_price * Decimal::from(100))
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use hypersdk::Decimal;

    use super::{PricePoint, PriceWindow};

    fn point(price: i64, seconds: u64) -> PricePoint {
        PricePoint::new(
            Decimal::from(price),
            UNIX_EPOCH + Duration::from_secs(seconds),
        )
    }

    #[test]
    fn keeps_points_inside_duration() {
        let mut window = PriceWindow::new(Duration::from_secs(60), 100);

        window.push(point(100, 0));
        window.push(point(101, 30));
        window.push(point(102, 61));

        assert_eq!(window.oldest(), Some(&point(101, 30)));
        assert_eq!(window.newest(), Some(&point(102, 61)));
    }

    #[test]
    fn enforces_maximum_point_count() {
        let mut window = PriceWindow::new(Duration::from_secs(60), 2);

        window.push(point(100, 0));
        window.push(point(101, 1));
        window.push(point(102, 2));

        assert_eq!(window.oldest(), Some(&point(101, 1)));
        assert_eq!(window.newest(), Some(&point(102, 2)));
    }

    #[test]
    fn ignores_out_of_order_points() {
        let mut window = PriceWindow::new(Duration::from_secs(60), 100);

        window.push(point(101, 10));
        window.push(point(100, 5));

        assert_eq!(window.oldest(), Some(&point(101, 10)));
        assert_eq!(window.newest(), Some(&point(101, 10)));
    }

    #[test]
    fn calculates_percentage_change_between_oldest_and_newest() {
        let mut window = PriceWindow::new(Duration::from_secs(60), 100);

        window.push(point(100, 0));
        window.push(point(105, 30));

        assert_eq!(window.percentage_change(), Some(Decimal::from(5)));
    }

    #[test]
    fn percentage_change_requires_two_points_and_nonzero_start() {
        let mut window = PriceWindow::new(Duration::from_secs(60), 100);
        assert_eq!(window.percentage_change(), None);

        window.push(point(100, 0));
        assert_eq!(window.percentage_change(), None);

        let mut zero_start = PriceWindow::new(Duration::from_secs(60), 100);
        zero_start.push(point(0, 0));
        zero_start.push(point(100, 1));
        assert_eq!(zero_start.percentage_change(), None);
    }
}
