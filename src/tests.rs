#[cfg(test)]
mod tests {
    use crate::variable_stats;

    #[test]
    fn should_calculate_variable_stats_correctly() {
        let test_data = vec![
            (vec![1.0, 2.0, 3.0, 4.0, 5.0], 3.0, 1.41),
            (vec![1.0, 3.0, 5.0, 7.0, 9.0], 5.0, 2.83),
            (vec![1.0, 9.0, 1.0, 9.0, 1.0], 4.2, 3.92),
            (vec![1.0, 0.5, 0.7, 0.9, 0.6], 0.74, 0.19),
            (vec![200.0, 3.0, 24.0, 92.0, 111.0], 86.0, 69.84),
        ];
        for (data, avg, dev) in test_data {
            let (cavg, cdev) = variable_stats(&data);
            assert!(cavg - avg < 0.1);
            assert!(cdev - dev < 0.1);
        } 
    }
}
