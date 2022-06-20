use anyhow::{bail, ensure, Context, Result};

use clap::Parser;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(
    name = "My RPN program",
    version = "1.0.0",
    author = "Nouka",
    about = "Super awesome sample RPN calculator"
)]
struct Opts {
    // Sets the level of verbosity
    #[clap(short, long)]
    verbose: bool,

    // Formulas written in RPN
    #[clap(name = "FILE")]
    formula_file: Option<PathBuf>,
}

/**
 * RpnCalculator
 */
struct RpnCalculator(bool);

impl RpnCalculator {
    // コンストラクタ?自身の構造体のパラメータを初期化
    pub fn new(verbose: bool) -> Self {
        Self(verbose)
    }

    // 行をパースして計算を実行する
    pub fn eval(&self, formula: &str) -> Result<i32> {
        // 文字列を空白でパースしトークンのVecを取得
        let mut tokens = formula.split_whitespace().rev().collect::<Vec<_>>();

        // 計算を実行して返す
        self.eval_inner(&mut tokens)
    }

    // 計算処理
    fn eval_inner(&self, tokens: &mut Vec<&str>) -> Result<i32> {
        // スタックの生成
        let mut stack = Vec::new();
        let mut pos = 0;

        // トークンが取り出せなくなるまでループ
        while let Some(token) = tokens.pop() {
            pos += 1;
            // トークンが数値だった場合
            if let Ok(x) = token.parse::<i32>() {
                // スタックに保存
                stack.push(x);
            } else {
                // トークンが数値以外の場合は、スタックから数値を取り出す
                let y = stack.pop().context(format!("invalid syntax at {}", pos))?;
                let x = stack.pop().context(format!("invalid syntax at {}", pos))?;
                // 取り出した数値をトークンの種類に応じて計算
                let res = match token {
                    "+" => x + y,
                    "-" => x - y,
                    "*" => x * y,
                    "/" => x / y,
                    "%" => x % y,
                    _ => bail!("invalid token at {}", pos),
                };
                // 計算結果をスタックに保存
                stack.push(res);
            }

            // `-v` オプションが指定されている場合は、この時点でのトークンとスタックの状態を出力
            if self.0 {
                println!("{:?} {:?}", tokens, stack);
            }
        }

        // スタックにデータが複数残っている場合はエラー
        ensure!(stack.len() == 1, "invalid syntax");

        Ok(stack[0])
    }
}

/**
 * メイン処理
 */
fn main() -> Result<()> {
    // Clapで提供された構造体を使ってコマンドライン引数を取得
    let opts = Opts::parse();

    // コマンドに渡されたのがファイルだった場合
    if let Some(path) = opts.formula_file {
        // ファイルをオープンしハンドラを取得
        let f = File::open(path)?;
        // ハンドラからリーダーを取得
        let reader = BufReader::new(f);
        run(reader, opts.verbose)
    } else {
        // コマンドに標準入力が渡された場合
        let stdin = stdin();
        // 標準入力からリーダーを取得
        let reader = stdin.lock();
        run(reader, opts.verbose)
    }
}

/**
 * リーダーで行を取得し計算を実行する処理
 */
fn run<R: BufRead>(reader: R, verbose: bool) -> Result<()> {
    // RpnCalculator のインスタンスを得る
    let calc = RpnCalculator::new(verbose);

    // リーダーを使って1行ずつ処理
    for line in reader.lines() {
        // 行を取得
        let line = line?;
        // 計算の実行
        match calc.eval(&line) {
            Ok(answer) => println!("{}", answer),
            Err(e) => eprintln!("{:#?}", e),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok() {
        let calc = RpnCalculator::new(false);
        assert_eq!(calc.eval("5").unwrap(), 5);
        assert_eq!(calc.eval("50").unwrap(), 50);
        assert_eq!(calc.eval("-50").unwrap(), -50);

        assert_eq!(calc.eval("2 3 +").unwrap(), 5);
        assert_eq!(calc.eval("2 3 *").unwrap(), 6);
        assert_eq!(calc.eval("2 3 -").unwrap(), -1);
        assert_eq!(calc.eval("2 3 /").unwrap(), 0);
        assert_eq!(calc.eval("2 3 %").unwrap(), 2);
    }

    #[test]
    fn test_ng() {
        let calc = RpnCalculator::new(false);
        assert!(calc.eval("").is_err());
        assert!(calc.eval("1 1 1 +").is_err());
        assert!(calc.eval("+ 1 1").is_err());
    }
}
