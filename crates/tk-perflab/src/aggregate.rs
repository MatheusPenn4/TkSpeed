//! PL-S0202 — motor de agregação estatística (reutilizável).
//! Hoje: média/desvio/min/max. Futuro (FPS): percentis (P99/P99.9) entram aqui.

#[derive(Debug, Clone)]
pub struct Aggregate {
    pub mean: f64,
    pub stddev: f64, // desvio-padrão amostral (n-1)
    pub min: f64,
    pub max: f64,
    pub samples: u32,
}

/// Agrega uma série de valores (uma métrica ao longo dos runs).
pub fn aggregate(values: &[f64]) -> Aggregate {
    if values.is_empty() {
        return Aggregate { mean: 0.0, stddev: 0.0, min: 0.0, max: 0.0, samples: 0 };
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let stddev = if values.len() > 1 {
        let var = values.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (n - 1.0);
        var.sqrt()
    } else {
        0.0
    };
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    Aggregate { mean, stddev, min, max, samples: values.len() as u32 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_basic() {
        let a = aggregate(&[10.0, 12.0, 11.0, 9.0, 13.0]);
        assert_eq!(a.samples, 5);
        assert!((a.mean - 11.0).abs() < 1e-9);
        assert!(a.stddev > 0.0);
        assert_eq!(a.min, 9.0);
        assert_eq!(a.max, 13.0);
    }

    #[test]
    fn aggregate_single_has_zero_stddev() {
        let a = aggregate(&[42.0]);
        assert_eq!(a.mean, 42.0);
        assert_eq!(a.stddev, 0.0);
    }

    #[test]
    fn aggregate_empty_is_safe() {
        let a = aggregate(&[]);
        assert_eq!(a.samples, 0);
        assert_eq!(a.mean, 0.0);
    }
}
