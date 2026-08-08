// src/bin/common/metrics.rs

//! Shared evaluation metrics for all training/evaluation binaries

use code_intelligence::analysis::training_data::{TrainingExample, TrainingLabel};
use code_intelligence::ml::classifier::DeadCodeClassifier;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EvaluationMetrics {
    pub total: usize,
    pub correct: usize,
    pub true_positives: usize,
    pub true_negatives: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    pub fpr: f64, // False Positive Rate
    pub fnr: f64, // False Negative Rate
    pub specificity: f64,
}

#[allow(dead_code)]
impl EvaluationMetrics {
    pub fn print(&self) {
        println!("   Total: {}", self.total);
        println!("   Correct: {}", self.correct);
        println!("   Confusion Matrix (DEAD = Positive Class):");
        println!("                   ACTUAL");
        println!("              Alive    Dead");
        println!(
            "   Pred Alive   {:>4}    {:>4}  ← False Negatives",
            self.true_negatives, self.false_negatives
        );
        println!(
            "   Pred Dead    {:>4}    {:>4}  ← True Positives",
            self.false_positives, self.true_positives
        );
        println!("\n   Metrics (Positive = DEAD):");
        println!("   Accuracy: {:.1}%", self.accuracy * 100.0);
        println!("   Precision: {:.1}%", self.precision * 100.0);
        println!("   Recall: {:.1}%", self.recall * 100.0);
        println!("   F1: {:.1}%", self.f1 * 100.0);
        println!("   FPR: {:.1}%", self.fpr * 100.0);
        println!("   FNR: {:.1}%", self.fnr * 100.0);
        println!("   Specificity: {:.1}%", self.specificity * 100.0);
    }
}

#[allow(dead_code)]
pub fn evaluate(
    classifier: &DeadCodeClassifier,
    examples: &[TrainingExample],
) -> EvaluationMetrics {
    let mut tp = 0;
    let mut tn = 0;
    let mut fp = 0;
    let mut fn_ = 0;

    for example in examples {
        let prediction = classifier.predict(example);
        let actual = &example.label;

        // DEAD is the positive class
        match (prediction, actual) {
            // True Positive: predicted Dead, actually Dead
            (TrainingLabel::Dead, TrainingLabel::Dead) => tp += 1,
            // True Negative: predicted Alive, actually Alive
            (TrainingLabel::Alive, TrainingLabel::Alive) => tn += 1,
            // False Negative: predicted Alive, actually Dead
            (TrainingLabel::Alive, TrainingLabel::Dead) => fn_ += 1,
            // False Positive: predicted Dead, actually Alive
            (TrainingLabel::Dead, TrainingLabel::Alive) => fp += 1,
            _ => {}
        }
    }

    let total = tp + tn + fp + fn_;
    let correct = tp + tn;

    let accuracy = if total > 0 {
        correct as f64 / total as f64
    } else {
        0.0
    };
    // DEAD is positive class, so:
    // Precision = TP / (TP + FP) - of all DEAD predictions, how many were actually DEAD?
    // Recall = TP / (TP + FN) - of all actual DEAD, how many did we find?
    let precision = if tp + fp > 0 {
        tp as f64 / (tp + fp) as f64
    } else {
        0.0
    };
    let recall = if tp + fn_ > 0 {
        tp as f64 / (tp + fn_) as f64
    } else {
        0.0
    };
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };
    let fpr = if fp + tn > 0 {
        fp as f64 / (fp + tn) as f64
    } else {
        0.0
    };
    let fnr = if fn_ + tp > 0 {
        fn_ as f64 / (fn_ + tp) as f64
    } else {
        0.0
    };
    let specificity = 1.0 - fpr;

    EvaluationMetrics {
        total,
        correct,
        true_positives: tp,
        true_negatives: tn,
        false_positives: fp,
        false_negatives: fn_,
        accuracy,
        precision,
        recall,
        f1,
        fpr,
        fnr,
        specificity,
    }
}
