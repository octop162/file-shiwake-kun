// CLI tool for testing file-shiwake-kun functionality
use file_shiwake_kun_lib::models::Config;
use file_shiwake_kun_lib::services::{
    ConfigManager, FileProcessor, RuleEngine, 
    DefaultMetadataExtractor, DefaultFileOperations, MetadataExtractor
};
use std::path::PathBuf;
use std::env;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }

    let command = &args[1];
    
    match command.as_str() {
        "process" => {
            if args.len() < 3 {
                eprintln!("使用法: cli process <ファイルパス> [ファイルパス...]");
                return;
            }
            let files: Vec<String> = args[2..].to_vec();
            process_files(files);
        },
        "load-config" => {
            load_config();
        },
        "save-config" => {
            if args.len() < 3 {
                eprintln!("使用法: cli save-config <設定ファイルパス>");
                return;
            }
            save_config(&args[2]);
        },
        "create-default-config" => {
            create_default_config();
        },
        "extract-metadata" => {
            if args.len() < 3 {
                eprintln!("使用法: cli extract-metadata <ファイルパス>");
                return;
            }
            extract_metadata(&args[2]);
        },
        "test-rule" => {
            if args.len() < 3 {
                eprintln!("使用法: cli test-rule <ファイルパス>");
                return;
            }
            test_rule(&args[2]);
        },
        "help" | "--help" | "-h" => {
            print_usage();
        },
        _ => {
            eprintln!("不明なコマンド: {}", command);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("ファイル仕訳け君 CLI ツール");
    println!();
    println!("使用法:");
    println!("  cli <コマンド> [引数...]");
    println!();
    println!("コマンド:");
    println!("  process <ファイル...>           - ファイルを処理する");
    println!("  load-config                     - 設定を読み込んで表示する");
    println!("  save-config <パス>              - デフォルト設定を指定パスに保存する");
    println!("  create-default-config           - デフォルト設定を作成する");
    println!("  extract-metadata <ファイル>     - ファイルのメタデータを抽出する");
    println!("  test-rule <ファイル>            - ファイルに対するルールマッチングをテストする");
    println!("  help                            - このヘルプを表示する");
    println!();
    println!("例:");
    println!("  cli process photo.jpg");
    println!("  cli extract-metadata photo.jpg");
    println!("  cli load-config");
}

fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("file-shiwake-kun");
    
    config_dir.join("config.toml")
}

fn process_files(files: Vec<String>) {
    println!("=== ファイル処理 ===");
    println!("処理するファイル数: {}", files.len());
    
    // Load configuration
    let config_manager = ConfigManager::new(get_config_path());
    let config = match config_manager.load() {
        Ok(cfg) => {
            println!("✓ 設定を読み込みました ({} ルール)", cfg.rules.len());
            cfg
        },
        Err(e) => {
            eprintln!("✗ 設定の読み込みに失敗: {}", e);
            return;
        }
    };
    
    // Create components
    let rule_engine = RuleEngine::new(config.rules.clone());
    let metadata_extractor = Box::new(DefaultMetadataExtractor);
    let file_ops = Box::new(DefaultFileOperations);
    
    let mut processor = FileProcessor::new(rule_engine, metadata_extractor, file_ops);
    
    if !config.default_destination.is_empty() {
        processor.set_default_destination(config.default_destination.clone());
    }
    
    processor.set_preview_mode(config.preview_mode);
    println!("プレビューモード: {}", config.preview_mode);
    println!();
    
    // Process files
    let results = processor.process_files(files);
    
    // Display results
    println!("=== 処理結果 ===");
    let success_count = results.iter().filter(|r| r.success).count();
    let failure_count = results.iter().filter(|r| !r.success).count();
    
    println!("成功: {} / 失敗: {} / 合計: {}", success_count, failure_count, results.len());
    println!();
    
    for result in &results {
        if result.success {
            println!("✓ {}", result.source_path);
            if let Some(dest) = &result.destination_path {
                println!("  → {}", dest);
            }
            if let Some(rule) = &result.matched_rule {
                println!("  ルール: {}", rule);
            }
        } else {
            println!("✗ {}", result.source_path);
            if let Some(err) = &result.error_message {
                println!("  エラー: {}", err);
            }
        }
        println!();
    }
}

fn load_config() {
    println!("=== 設定の読み込み ===");
    
    let config_path = get_config_path();
    println!("設定ファイル: {}", config_path.display());
    
    let config_manager = ConfigManager::new(config_path);
    match config_manager.load() {
        Ok(config) => {
            println!("✓ 設定を読み込みました");
            println!();
            println!("デフォルト移動先: {}", config.default_destination);
            println!("プレビューモード: {}", config.preview_mode);
            println!("ログパス: {}", config.log_path);
            println!();
            println!("ルール数: {}", config.rules.len());
            println!();
            
            for (i, rule) in config.rules.iter().enumerate() {
                println!("ルール {}: {}", i + 1, rule.name);
                println!("  ID: {}", rule.id);
                println!("  優先度: {}", rule.priority);
                println!("  操作: {:?}", rule.operation);
                println!("  移動先パターン: {}", rule.destination_pattern);
                println!("  条件数: {}", rule.conditions.len());
                for (j, cond) in rule.conditions.iter().enumerate() {
                    println!("    条件 {}: {} {} {:?}", j + 1, cond.field, cond.operator, cond.value);
                }
                println!();
            }
        },
        Err(e) => {
            eprintln!("✗ 設定の読み込みに失敗: {}", e);
        }
    }
}

fn save_config(path: &str) {
    println!("=== 設定の保存 ===");
    println!("保存先: {}", path);
    
    let config = Config {
        rules: vec![],
        default_destination: String::from(""),
        preview_mode: false,
        log_path: String::from("file-shiwake-kun.log"),
    };
    
    let config_manager = ConfigManager::new(PathBuf::from(path));
    match config_manager.save(&config) {
        Ok(_) => {
            println!("✓ デフォルト設定を保存しました");
        },
        Err(e) => {
            eprintln!("✗ 設定の保存に失敗: {}", e);
        }
    }
}

fn create_default_config() {
    println!("=== デフォルト設定の作成 ===");
    
    let config_path = get_config_path();
    println!("設定ファイル: {}", config_path.display());
    
    let config_manager = ConfigManager::new(config_path);
    let config = Config {
        rules: vec![],
        default_destination: String::from(""),
        preview_mode: false,
        log_path: String::from("file-shiwake-kun.log"),
    };
    
    match config_manager.save(&config) {
        Ok(_) => {
            println!("✓ デフォルト設定を作成しました");
            println!();
            println!("デフォルト移動先: {}", config.default_destination);
            println!("プレビューモード: {}", config.preview_mode);
            println!("ルール数: {}", config.rules.len());
        },
        Err(e) => {
            eprintln!("✗ 設定の作成に失敗: {}", e);
        }
    }
}

fn extract_metadata(file_path: &str) {
    println!("=== メタデータ抽出 ===");
    println!("ファイル: {}", file_path);
    println!();
    
    let extractor = DefaultMetadataExtractor;
    match extractor.extract(file_path) {
        Ok(metadata) => {
            println!("✓ メタデータを抽出しました");
            println!();
            println!("ファイル名: {}", metadata.filename);
            println!("拡張子: {}", metadata.extension);
            println!("サイズ: {} bytes", metadata.size);
            
            if let Some(created) = metadata.created_at {
                println!("作成日時: {:?}", created);
            }
            println!("更新日時: {:?}", metadata.modified_at);
            
            if let Some(capture_date) = metadata.capture_date {
                println!();
                println!("=== EXIF情報 ===");
                println!("撮影日時: {:?}", capture_date);
            }
            
            if let Some(camera) = metadata.camera_model {
                println!("カメラ機種: {}", camera);
            }
            
            if let Some(lat) = metadata.gps_latitude {
                println!("GPS緯度: {}", lat);
            }
            
            if let Some(lon) = metadata.gps_longitude {
                println!("GPS経度: {}", lon);
            }
        },
        Err(e) => {
            eprintln!("✗ メタデータの抽出に失敗: {}", e);
        }
    }
}

fn test_rule(file_path: &str) {
    println!("=== ルールマッチングテスト ===");
    println!("ファイル: {}", file_path);
    println!();
    
    // Load configuration
    let config_manager = ConfigManager::new(get_config_path());
    let config = match config_manager.load() {
        Ok(cfg) => {
            println!("✓ 設定を読み込みました ({} ルール)", cfg.rules.len());
            cfg
        },
        Err(e) => {
            eprintln!("✗ 設定の読み込みに失敗: {}", e);
            return;
        }
    };
    
    // Extract metadata
    let extractor = DefaultMetadataExtractor;
    let metadata = match extractor.extract(file_path) {
        Ok(m) => {
            println!("✓ メタデータを抽出しました");
            m
        },
        Err(e) => {
            eprintln!("✗ メタデータの抽出に失敗: {}", e);
            return;
        }
    };
    
    println!();
    println!("ファイル情報:");
    println!("  拡張子: {}", metadata.extension);
    println!("  サイズ: {} bytes", metadata.size);
    
    // Test rules
    let rule_engine = RuleEngine::new(config.rules);
    
    println!();
    println!("=== ルールマッチング ===");
    
    match rule_engine.find_matching_rule(&metadata) {
        Some(rule) => {
            println!("✓ マッチしたルール: {}", rule.name);
            println!("  優先度: {}", rule.priority);
            println!("  操作: {:?}", rule.operation);
            println!("  移動先パターン: {}", rule.destination_pattern);
            
            match rule_engine.apply_rule(rule, &metadata) {
                Ok(dest_path) => {
                    println!();
                    println!("移動先パス: {}", dest_path);
                },
                Err(e) => {
                    eprintln!("✗ ルールの適用に失敗: {}", e);
                }
            }
        },
        None => {
            println!("✗ マッチするルールがありません");
        }
    }
}
