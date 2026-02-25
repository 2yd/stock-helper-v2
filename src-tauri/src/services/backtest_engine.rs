use crate::models::backtest::*;
use crate::models::watchlist::KlineItem;
use std::collections::HashMap;

/// 持仓信息
struct Position {
    code: String,
    name: String,
    shares: f64,
    cost_price: f64,
    open_date: String,
}

/// 执行回测
pub fn run_backtest(
    config: &BacktestConfig,
    kline_map: &HashMap<String, Vec<KlineItem>>,
    benchmark_klines: &[KlineItem],
) -> BacktestResult {
    let mut cash = config.initial_capital;
    let mut positions: HashMap<String, Position> = HashMap::new();
    let mut trades: Vec<BacktestTrade> = Vec::new();
    let mut equity_curve: Vec<EquityPoint> = Vec::new();
    let mut trade_id = 0u32;

    // 建立基准净值映射
    let benchmark_map: HashMap<&str, f64> = benchmark_klines.iter()
        .map(|k| (k.date.as_str(), k.close))
        .collect();
    let benchmark_start = benchmark_klines.first().map(|k| k.close).unwrap_or(1.0);

    // 收集所有交易日
    let mut all_dates: Vec<String> = Vec::new();
    for klines in kline_map.values() {
        for k in klines {
            if k.date >= config.start_date && k.date <= config.end_date && !all_dates.contains(&k.date) {
                all_dates.push(k.date.clone());
            }
        }
    }
    all_dates.sort();

    let max_position_value = config.initial_capital * config.max_position_pct;
    let num_codes = config.codes.len().max(1);

    // 逐日遍历
    for date in &all_dates {
        // 检查卖出条件
        let mut sell_codes: Vec<String> = Vec::new();
        for (code, pos) in &positions {
            if let Some(klines) = kline_map.get(code) {
                if let Some(today) = klines.iter().find(|k| &k.date == date) {
                    let pnl_pct = (today.close - pos.cost_price) / pos.cost_price;

                    // 止损
                    if pnl_pct < -config.stop_loss {
                        sell_codes.push(code.clone());
                        continue;
                    }
                    // 止盈
                    if pnl_pct > config.take_profit {
                        sell_codes.push(code.clone());
                        continue;
                    }

                    // 基于简单动量的卖出判断
                    let idx = klines.iter().position(|k| &k.date == date).unwrap_or(0);
                    if idx >= 5 {
                        let ma5: f64 = klines[idx-4..=idx].iter().map(|k| k.close).sum::<f64>() / 5.0;
                        if today.close < ma5 * 0.97 {
                            sell_codes.push(code.clone());
                        }
                    }
                }
            }
        }

        // 执行卖出
        for code in &sell_codes {
            if let Some(pos) = positions.remove(code) {
                if let Some(klines) = kline_map.get(code) {
                    if let Some(today) = klines.iter().find(|k| &k.date == date) {
                        let sell_price = today.close * (1.0 - config.slippage);
                        let gross = sell_price * pos.shares;
                        let commission = gross * config.commission_rate;
                        let net_profit = (sell_price - pos.cost_price) * pos.shares - commission;
                        let profit_pct = (sell_price - pos.cost_price) / pos.cost_price;

                        let holding_days = count_trading_days(&all_dates, &pos.open_date, date);

                        cash += gross - commission;
                        trade_id += 1;
                        trades.push(BacktestTrade {
                            id: trade_id,
                            code: code.clone(),
                            name: pos.name.clone(),
                            direction: "sell".into(),
                            open_date: pos.open_date.clone(),
                            open_price: pos.cost_price,
                            close_date: date.clone(),
                            close_price: sell_price,
                            shares: pos.shares,
                            profit: net_profit,
                            profit_pct,
                            holding_days,
                            commission,
                        });
                    }
                }
            }
        }

        // 检查买入条件
        for code in &config.codes {
            if positions.contains_key(code) { continue; }
            if let Some(klines) = kline_map.get(code) {
                let idx = klines.iter().position(|k| &k.date == date);
                if let Some(idx) = idx {
                    if idx < 20 { continue; } // 需要足够的历史数据

                    // 简单买入信号：MA5 > MA20 且 MA5 刚上穿 MA20
                    let ma5_now: f64 = klines[idx-4..=idx].iter().map(|k| k.close).sum::<f64>() / 5.0;
                    let ma20_now: f64 = klines[idx-19..=idx].iter().map(|k| k.close).sum::<f64>() / 20.0;
                    let ma5_prev: f64 = klines[idx-5..=idx-1].iter().map(|k| k.close).sum::<f64>() / 5.0;
                    let ma20_prev: f64 = klines[idx-20..=idx-1].iter().map(|k| k.close).sum::<f64>() / 20.0;

                    let golden_cross = ma5_prev <= ma20_prev && ma5_now > ma20_now;
                    let uptrend = klines[idx].close > ma20_now;

                    if golden_cross && uptrend {
                        let buy_price = klines[idx].close * (1.0 + config.slippage);
                        let available = cash.min(max_position_value / num_codes as f64);
                        if available < buy_price * 100.0 { continue; } // 至少买1手

                        let shares = (available / buy_price / 100.0).floor() * 100.0;
                        if shares <= 0.0 { continue; }

                        let cost = buy_price * shares;
                        let commission = cost * config.commission_rate;
                        cash -= cost + commission;

                        let name = code.clone(); // 简化，实际可传入名称映射
                        positions.insert(code.clone(), Position {
                            code: code.clone(),
                            name,
                            shares,
                            cost_price: buy_price,
                            open_date: date.clone(),
                        });
                    }
                }
            }
        }

        // 计算当日总权益
        let mut total_equity = cash;
        for (code, pos) in &positions {
            if let Some(klines) = kline_map.get(code) {
                if let Some(today) = klines.iter().find(|k| &k.date == date) {
                    total_equity += today.close * pos.shares;
                } else {
                    total_equity += pos.cost_price * pos.shares;
                }
            }
        }

        let benchmark_value = benchmark_map.get(date.as_str()).copied().unwrap_or(benchmark_start);
        let benchmark_nav = benchmark_value / benchmark_start;

        equity_curve.push(EquityPoint {
            date: date.clone(),
            equity: total_equity / config.initial_capital,
            benchmark: benchmark_nav,
            drawdown: 0.0,
        });
    }

    // 强制平仓剩余持仓
    let remaining: Vec<String> = positions.keys().cloned().collect();
    for code in remaining {
        if let Some(pos) = positions.remove(&code) {
            if let Some(klines) = kline_map.get(&code) {
                if let Some(last) = klines.last() {
                    let sell_price = last.close;
                    let gross = sell_price * pos.shares;
                    let commission = gross * config.commission_rate;
                    let net_profit = (sell_price - pos.cost_price) * pos.shares - commission;
                    let profit_pct = (sell_price - pos.cost_price) / pos.cost_price;
                    let holding_days = count_trading_days(&all_dates, &pos.open_date, &last.date);

                    trade_id += 1;
                    trades.push(BacktestTrade {
                        id: trade_id,
                        code: code.clone(),
                        name: pos.name.clone(),
                        direction: "sell".into(),
                        open_date: pos.open_date.clone(),
                        open_price: pos.cost_price,
                        close_date: last.date.clone(),
                        close_price: sell_price,
                        shares: pos.shares,
                        profit: net_profit,
                        profit_pct,
                        holding_days,
                        commission,
                    });
                }
            }
        }
    }

    // 计算回撤
    compute_drawdowns(&mut equity_curve);

    // 计算绩效指标
    let performance = compute_performance(&equity_curve, &trades, &all_dates);

    BacktestResult {
        config: config.clone(),
        performance,
        equity_curve,
        trades,
    }
}

fn compute_drawdowns(curve: &mut Vec<EquityPoint>) {
    let mut peak = 0.0_f64;
    for point in curve.iter_mut() {
        if point.equity > peak {
            peak = point.equity;
        }
        point.drawdown = if peak > 0.0 { (peak - point.equity) / peak } else { 0.0 };
    }
}

fn compute_performance(
    curve: &[EquityPoint],
    trades: &[BacktestTrade],
    all_dates: &[String],
) -> BacktestPerformance {
    if curve.is_empty() {
        return BacktestPerformance::default();
    }

    let final_nav = curve.last().unwrap().equity;
    let total_return = final_nav - 1.0;

    let trading_days = all_dates.len() as f64;
    let years = trading_days / 250.0;
    let annual_return = if years > 0.0 { final_nav.powf(1.0 / years) - 1.0 } else { 0.0 };

    let max_drawdown = curve.iter().map(|p| p.drawdown).fold(0.0_f64, f64::max);

    // Sharpe ratio (assuming risk-free rate = 3%)
    let daily_returns: Vec<f64> = curve.windows(2).map(|w| w[1].equity / w[0].equity - 1.0).collect();
    let avg_return = if !daily_returns.is_empty() { daily_returns.iter().sum::<f64>() / daily_returns.len() as f64 } else { 0.0 };
    let variance = if daily_returns.len() > 1 {
        daily_returns.iter().map(|r| (r - avg_return).powi(2)).sum::<f64>() / (daily_returns.len() - 1) as f64
    } else { 0.0 };
    let std_dev = variance.sqrt();
    let sharpe_ratio = if std_dev > 0.0 { (annual_return - 0.03) / (std_dev * 250.0_f64.sqrt()) } else { 0.0 };

    let winning_trades = trades.iter().filter(|t| t.profit > 0.0).count() as u32;
    let losing_trades = trades.iter().filter(|t| t.profit <= 0.0).count() as u32;
    let total_trades = trades.len() as u32;
    let win_rate = if total_trades > 0 { winning_trades as f64 / total_trades as f64 } else { 0.0 };

    let avg_win: f64 = {
        let wins: Vec<f64> = trades.iter().filter(|t| t.profit > 0.0).map(|t| t.profit).collect();
        if !wins.is_empty() { wins.iter().sum::<f64>() / wins.len() as f64 } else { 0.0 }
    };
    let avg_loss: f64 = {
        let losses: Vec<f64> = trades.iter().filter(|t| t.profit <= 0.0).map(|t| t.profit.abs()).collect();
        if !losses.is_empty() { losses.iter().sum::<f64>() / losses.len() as f64 } else { 1.0 }
    };
    let profit_loss_ratio = if avg_loss > 0.0 { avg_win / avg_loss } else { 0.0 };

    // Consecutive wins/losses
    let (max_consecutive_wins, max_consecutive_losses) = {
        let mut max_w = 0u32; let mut max_l = 0u32;
        let mut cur_w = 0u32; let mut cur_l = 0u32;
        for t in trades {
            if t.profit > 0.0 { cur_w += 1; cur_l = 0; max_w = max_w.max(cur_w); }
            else { cur_l += 1; cur_w = 0; max_l = max_l.max(cur_l); }
        }
        (max_w, max_l)
    };

    let avg_holding_days = if total_trades > 0 {
        trades.iter().map(|t| t.holding_days as f64).sum::<f64>() / total_trades as f64
    } else { 0.0 };

    let benchmark_return = curve.last().map(|p| p.benchmark - 1.0).unwrap_or(0.0);
    let alpha = annual_return - benchmark_return / years.max(1.0);

    BacktestPerformance {
        total_return,
        annual_return,
        max_drawdown,
        sharpe_ratio,
        win_rate,
        profit_loss_ratio,
        total_trades,
        winning_trades,
        losing_trades,
        max_consecutive_wins,
        max_consecutive_losses,
        avg_holding_days,
        benchmark_return,
        alpha,
    }
}

fn count_trading_days(all_dates: &[String], start: &str, end: &str) -> u32 {
    let s = all_dates.iter().position(|d| d == start).unwrap_or(0);
    let e = all_dates.iter().position(|d| d == end).unwrap_or(s);
    (e - s) as u32
}
