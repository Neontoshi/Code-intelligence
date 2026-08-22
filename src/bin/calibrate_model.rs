// src/bin/calibrate_model.rs

//! Calibrate a trained model using validation data

use clap::Parser;
use code_intelligence::analysis::training_data::TrainingExample;
use code_intelligence::ml::calibration::CalibratedModel;
use code_intelligence::ml::calibration::CalibrationMethod;
use code_intelligence::ml::classifier::DeadCodeClassifier;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    /// Model file path
    #[arg(short, long, default_value = "model.bin")]
    model: PathBuf,

    /// Validation data file
    #[arg(short, long, default_value = "data/val.json")]
    val_data: PathBuf,

    /// Output model path
    #[arg(short, long, default_value = "model_calibrated.bin")]
    output: PathBuf,

    /// Calibration method: temperature, histogram, isotonic
    #[arg(long, default_value = "temperature")]
    method: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🔬 Model Calibration");
    println!("===================\n");

    // Load model
    println!("📊 Loading model from: {:?}", args.model);
    let classifier = DeadCodeClassifier::load(&args.model.to_string_lossy())?;

    let model = classifier.model.ok_or("No model found")?;

    // Load validation data
    println!("📊 Loading validation data from: {:?}", args.val_data);
    let data = std::fs::read_to_string(&args.val_data)?;
    let val_examples: Vec<TrainingExample> = serde_json::from_str(&data)?;

    println!("   Validation examples: {}", val_examples.len());

    // Parse method
    let method = match args.method.as_str() {
        "temperature" => CalibrationMethod::TemperatureScaling,
        "histogram" => CalibrationMethod::HistogramBinning,
        "none" => CalibrationMethod::None,
        _ => {
            eprintln!("Unknown method: {}", args.method);
            eprintln!("Available: temperature, histogram, none");
            std::process::exit(1);
        }
    };

    // Calibrate
    println!("\n🧪 Calibrating with {:?}...", method);
    let calibrated = CalibratedModel::calibrate(&model, &val_examples, method);

    // Show stats
    let stats = calibrated.calibration_stats(&val_examples);
    println!("\n📊 Calibration Statistics:");
    stats.print();

    let mut new_classifier = DeadCodeClassifier::new();
    new_classifier.model = Some(calibrated.classifier);
    new_classifier.save(&args.output.to_string_lossy())?;
    println!("\n✅ Calibrated model saved to: {:?}", args.output);

    Ok(())
}
