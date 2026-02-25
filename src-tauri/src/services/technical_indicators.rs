use crate::models::watchlist::{
    KlineItem, MaAlignment, TechnicalIndicators, TechnicalSignal, VolumePriceRelation,
};

/// 计算所有技术指标
pub fn compute_indicators(klines: &[KlineItem]) -> TechnicalIndicators {
    let closes: Vec<f64> = klines.iter().map(|k| k.close).collect();
    let highs: Vec<f64> = klines.iter().map(|k| k.high).collect();
    let lows: Vec<f64> = klines.iter().map(|k| k.low).collect();
    let volumes: Vec<f64> = klines.iter().map(|k| k.volume).collect();
    let dates: Vec<String> = klines.iter().map(|k| k.date.clone()).collect();

    let ma5 = calc_ma(&closes, 5);
    let ma10 = calc_ma(&closes, 10);
    let ma20 = calc_ma(&closes, 20);
    let ma60 = calc_ma(&closes, 60);

    let ema12 = calc_ema(&closes, 12);
    let ema26 = calc_ema(&closes, 26);
    let (macd_dif, macd_dea, macd_hist) = calc_macd(&closes, 12, 26, 9);
    let (kdj_k, kdj_d, kdj_j) = calc_kdj(&highs, &lows, &closes, 9, 3, 3);
    let rsi6 = calc_rsi(&closes, 6);
    let rsi12 = calc_rsi(&closes, 12);
    let rsi24 = calc_rsi(&closes, 24);
    let (boll_upper, boll_middle, boll_lower) = calc_boll(&closes, 20, 2.0);

    let _ = volumes; // used in signal detection

    TechnicalIndicators {
        dates,
        ma5, ma10, ma20, ma60,
        ema12, ema26,
        macd_dif, macd_dea, macd_hist,
        kdj_k, kdj_d, kdj_j,
        rsi6, rsi12, rsi24,
        boll_upper, boll_middle, boll_lower,
    }
}

/// 检测技术信号
pub fn detect_signals(klines: &[KlineItem], indicators: &TechnicalIndicators) -> Vec<TechnicalSignal> {
    let mut signals = Vec::new();
    let n = klines.len();
    if n < 3 {
        return signals;
    }

    // 只检测最近5个交易日的信号
    let check_start = if n > 5 { n - 5 } else { 0 };

    for i in check_start..n {
        if i < 1 { continue; }

        // MA 金叉/死叉
        detect_ma_cross(&indicators.ma5, &indicators.ma20, i, "MA5/MA20", &klines[i].date, &mut signals);
        detect_ma_cross(&indicators.ma5, &indicators.ma10, i, "MA5/MA10", &klines[i].date, &mut signals);
        detect_ma_cross(&indicators.ma10, &indicators.ma20, i, "MA10/MA20", &klines[i].date, &mut signals);

        // MACD 金叉/死叉
        detect_ma_cross(&indicators.macd_dif, &indicators.macd_dea, i, "MACD", &klines[i].date, &mut signals);

        // KDJ 超买超卖
        if let (Some(k), Some(d)) = (indicators.kdj_k[i], indicators.kdj_d[i]) {
            if k > 80.0 && d > 80.0 {
                signals.push(TechnicalSignal {
                    signal_type: "kdj_overbought".into(),
                    direction: "bearish".into(),
                    description: format!("KDJ超买 K={:.1} D={:.1}", k, d),
                    strength: if k > 90.0 { 4 } else { 3 },
                    date: klines[i].date.clone(),
                });
            } else if k < 20.0 && d < 20.0 {
                signals.push(TechnicalSignal {
                    signal_type: "kdj_oversold".into(),
                    direction: "bullish".into(),
                    description: format!("KDJ超卖 K={:.1} D={:.1}", k, d),
                    strength: if k < 10.0 { 4 } else { 3 },
                    date: klines[i].date.clone(),
                });
            }
        }

        // RSI 超买超卖
        if let Some(rsi) = indicators.rsi6[i] {
            if rsi > 80.0 {
                signals.push(TechnicalSignal {
                    signal_type: "rsi_overbought".into(),
                    direction: "bearish".into(),
                    description: format!("RSI6超买 {:.1}", rsi),
                    strength: if rsi > 90.0 { 4 } else { 3 },
                    date: klines[i].date.clone(),
                });
            } else if rsi < 20.0 {
                signals.push(TechnicalSignal {
                    signal_type: "rsi_oversold".into(),
                    direction: "bullish".into(),
                    description: format!("RSI6超卖 {:.1}", rsi),
                    strength: if rsi < 10.0 { 4 } else { 3 },
                    date: klines[i].date.clone(),
                });
            }
        }

        // 布林带突破
        if let (Some(upper), Some(lower)) = (indicators.boll_upper[i], indicators.boll_lower[i]) {
            if klines[i].close > upper {
                signals.push(TechnicalSignal {
                    signal_type: "boll_break_upper".into(),
                    direction: "bearish".into(),
                    description: "股价突破布林带上轨".into(),
                    strength: 3,
                    date: klines[i].date.clone(),
                });
            } else if klines[i].close < lower {
                signals.push(TechnicalSignal {
                    signal_type: "boll_break_lower".into(),
                    direction: "bullish".into(),
                    description: "股价跌破布林带下轨".into(),
                    strength: 3,
                    date: klines[i].date.clone(),
                });
            }
        }

        // 放量信号
        if i >= 5 {
            let avg_vol: f64 = klines[i-5..i].iter().map(|k| k.volume).sum::<f64>() / 5.0;
            if avg_vol > 0.0 {
                let vol_ratio = klines[i].volume / avg_vol;
                if vol_ratio > 2.0 && klines[i].change_pct > 0.0 {
                    signals.push(TechnicalSignal {
                        signal_type: "volume_surge_up".into(),
                        direction: "bullish".into(),
                        description: format!("放量上攻 量比{:.1}", vol_ratio),
                        strength: if vol_ratio > 3.0 { 4 } else { 3 },
                        date: klines[i].date.clone(),
                    });
                } else if vol_ratio > 2.0 && klines[i].change_pct < -1.0 {
                    signals.push(TechnicalSignal {
                        signal_type: "volume_surge_down".into(),
                        direction: "bearish".into(),
                        description: format!("放量下跌 量比{:.1}", vol_ratio),
                        strength: if vol_ratio > 3.0 { 4 } else { 3 },
                        date: klines[i].date.clone(),
                    });
                }
            }
        }
    }

    // MACD 顶背离/底背离检测（最近30个交易日）
    detect_macd_divergence(klines, indicators, &mut signals);

    signals
}

/// 判断均线排列状态
pub fn determine_ma_alignment(indicators: &TechnicalIndicators) -> MaAlignment {
    let n = indicators.ma5.len();
    if n == 0 { return MaAlignment::Tangled; }
    let i = n - 1;

    match (indicators.ma5[i], indicators.ma10[i], indicators.ma20[i], indicators.ma60[i]) {
        (Some(ma5), Some(ma10), Some(ma20), Some(ma60)) => {
            if ma5 > ma10 && ma10 > ma20 && ma20 > ma60 {
                MaAlignment::Bullish
            } else if ma5 < ma10 && ma10 < ma20 && ma20 < ma60 {
                MaAlignment::Bearish
            } else {
                MaAlignment::Tangled
            }
        }
        (Some(ma5), Some(ma10), Some(ma20), None) => {
            if ma5 > ma10 && ma10 > ma20 {
                MaAlignment::Bullish
            } else if ma5 < ma10 && ma10 < ma20 {
                MaAlignment::Bearish
            } else {
                MaAlignment::Tangled
            }
        }
        _ => MaAlignment::Tangled,
    }
}

/// 判断量价关系
pub fn determine_volume_price_relation(klines: &[KlineItem]) -> VolumePriceRelation {
    let n = klines.len();
    if n < 6 { return VolumePriceRelation::Normal; }

    let recent_avg_vol: f64 = klines[n-3..n].iter().map(|k| k.volume).sum::<f64>() / 3.0;
    let prev_avg_vol: f64 = klines[n-6..n-3].iter().map(|k| k.volume).sum::<f64>() / 3.0;
    let recent_avg_change: f64 = klines[n-3..n].iter().map(|k| k.change_pct).sum::<f64>() / 3.0;

    if prev_avg_vol <= 0.0 { return VolumePriceRelation::Normal; }

    let vol_change = (recent_avg_vol - prev_avg_vol) / prev_avg_vol;

    if vol_change > 0.3 && recent_avg_change > 0.5 {
        VolumePriceRelation::VolumeUpPriceUp
    } else if vol_change < -0.3 && recent_avg_change > 0.5 {
        VolumePriceRelation::VolumeDownPriceUp
    } else if vol_change > 0.3 && recent_avg_change < -0.5 {
        VolumePriceRelation::VolumeUpPriceDown
    } else if vol_change < -0.3 && recent_avg_change < -0.5 {
        VolumePriceRelation::VolumeDownPriceDown
    } else {
        VolumePriceRelation::Normal
    }
}

/// 生成技术分析文字摘要
pub fn generate_summary(
    ma_alignment: &MaAlignment,
    volume_price: &VolumePriceRelation,
    signals: &[TechnicalSignal],
) -> String {
    let mut parts = Vec::new();

    match ma_alignment {
        MaAlignment::Bullish => parts.push("均线多头排列".to_string()),
        MaAlignment::Bearish => parts.push("均线空头排列".to_string()),
        MaAlignment::Tangled => parts.push("均线纠缠".to_string()),
    }

    match volume_price {
        VolumePriceRelation::VolumeUpPriceUp => parts.push("放量上涨".to_string()),
        VolumePriceRelation::VolumeDownPriceUp => parts.push("缩量上涨".to_string()),
        VolumePriceRelation::VolumeUpPriceDown => parts.push("放量下跌".to_string()),
        VolumePriceRelation::VolumeDownPriceDown => parts.push("缩量下跌".to_string()),
        VolumePriceRelation::Normal => parts.push("量价正常".to_string()),
    }

    let recent_signals: Vec<&TechnicalSignal> = signals.iter()
        .filter(|s| s.strength >= 3)
        .collect();

    for sig in recent_signals.iter().take(3) {
        parts.push(sig.description.clone());
    }

    parts.join("；")
}

// ====== 指标计算函数 ======

fn calc_ma(data: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = vec![None; data.len()];
    if data.len() < period { return result; }

    let mut sum: f64 = data[..period].iter().sum();
    result[period - 1] = Some(sum / period as f64);

    for i in period..data.len() {
        sum += data[i] - data[i - period];
        result[i] = Some(sum / period as f64);
    }
    result
}

fn calc_ema(data: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = vec![None; data.len()];
    if data.is_empty() || period == 0 { return result; }

    let multiplier = 2.0 / (period as f64 + 1.0);
    result[0] = Some(data[0]);

    for i in 1..data.len() {
        let prev = result[i - 1].unwrap_or(data[i]);
        result[i] = Some(data[i] * multiplier + prev * (1.0 - multiplier));
    }
    result
}

fn calc_macd(data: &[f64], fast: usize, slow: usize, signal: usize) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let ema_fast = calc_ema(data, fast);
    let ema_slow = calc_ema(data, slow);
    let n = data.len();

    let mut dif = vec![None; n];
    let mut dif_values = Vec::new();

    for i in 0..n {
        if let (Some(f), Some(s)) = (ema_fast[i], ema_slow[i]) {
            let d = f - s;
            dif[i] = Some(d);
            dif_values.push(d);
        }
    }

    // DEA = EMA(DIF, signal)
    let dea_raw = calc_ema(&dif_values, signal);
    let mut dea = vec![None; n];
    let mut hist = vec![None; n];
    let mut dea_idx = 0;

    for i in 0..n {
        if dif[i].is_some() {
            if dea_idx < dea_raw.len() {
                dea[i] = dea_raw[dea_idx];
                if let (Some(d), Some(de)) = (dif[i], dea[i]) {
                    hist[i] = Some((d - de) * 2.0);
                }
                dea_idx += 1;
            }
        }
    }

    (dif, dea, hist)
}

fn calc_kdj(highs: &[f64], lows: &[f64], closes: &[f64], n: usize, m1: usize, m2: usize) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let len = closes.len();
    let mut k_vals = vec![None; len];
    let mut d_vals = vec![None; len];
    let mut j_vals = vec![None; len];

    if len < n { return (k_vals, d_vals, j_vals); }

    let mut prev_k = 50.0_f64;
    let mut prev_d = 50.0_f64;

    for i in (n - 1)..len {
        let start = i + 1 - n;
        let highest = highs[start..=i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let lowest = lows[start..=i].iter().cloned().fold(f64::INFINITY, f64::min);

        let rsv = if (highest - lowest).abs() < 1e-10 {
            50.0
        } else {
            (closes[i] - lowest) / (highest - lowest) * 100.0
        };

        let k = prev_k * (m1 as f64 - 1.0) / m1 as f64 + rsv / m1 as f64;
        let d = prev_d * (m2 as f64 - 1.0) / m2 as f64 + k / m2 as f64;
        let j = 3.0 * k - 2.0 * d;

        k_vals[i] = Some(k);
        d_vals[i] = Some(d);
        j_vals[i] = Some(j);

        prev_k = k;
        prev_d = d;
    }

    (k_vals, d_vals, j_vals)
}

fn calc_rsi(data: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = vec![None; data.len()];
    if data.len() < period + 1 { return result; }

    let mut avg_gain = 0.0;
    let mut avg_loss = 0.0;

    for i in 1..=period {
        let change = data[i] - data[i - 1];
        if change > 0.0 { avg_gain += change; }
        else { avg_loss += change.abs(); }
    }

    avg_gain /= period as f64;
    avg_loss /= period as f64;

    if avg_loss.abs() < 1e-10 {
        result[period] = Some(100.0);
    } else {
        let rs = avg_gain / avg_loss;
        result[period] = Some(100.0 - 100.0 / (1.0 + rs));
    }

    for i in (period + 1)..data.len() {
        let change = data[i] - data[i - 1];
        let (gain, loss) = if change > 0.0 { (change, 0.0) } else { (0.0, change.abs()) };

        avg_gain = (avg_gain * (period as f64 - 1.0) + gain) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + loss) / period as f64;

        if avg_loss.abs() < 1e-10 {
            result[i] = Some(100.0);
        } else {
            let rs = avg_gain / avg_loss;
            result[i] = Some(100.0 - 100.0 / (1.0 + rs));
        }
    }

    result
}

fn calc_boll(data: &[f64], period: usize, multiplier: f64) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let n = data.len();
    let mut upper = vec![None; n];
    let mut middle = vec![None; n];
    let mut lower = vec![None; n];

    if n < period { return (upper, middle, lower); }

    for i in (period - 1)..n {
        let start = i + 1 - period;
        let slice = &data[start..=i];
        let mean: f64 = slice.iter().sum::<f64>() / period as f64;
        let variance: f64 = slice.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / period as f64;
        let std_dev = variance.sqrt();

        middle[i] = Some(mean);
        upper[i] = Some(mean + multiplier * std_dev);
        lower[i] = Some(mean - multiplier * std_dev);
    }

    (upper, middle, lower)
}

fn detect_ma_cross(fast: &[Option<f64>], slow: &[Option<f64>], i: usize, label: &str, date: &str, signals: &mut Vec<TechnicalSignal>) {
    if i < 1 { return; }
    if let (Some(f_now), Some(s_now), Some(f_prev), Some(s_prev)) = (fast[i], slow[i], fast[i-1], slow[i-1]) {
        if f_prev <= s_prev && f_now > s_now {
            signals.push(TechnicalSignal {
                signal_type: "golden_cross".into(),
                direction: "bullish".into(),
                description: format!("{} 金叉", label),
                strength: if label == "MACD" { 4 } else { 3 },
                date: date.to_string(),
            });
        } else if f_prev >= s_prev && f_now < s_now {
            signals.push(TechnicalSignal {
                signal_type: "death_cross".into(),
                direction: "bearish".into(),
                description: format!("{} 死叉", label),
                strength: if label == "MACD" { 4 } else { 3 },
                date: date.to_string(),
            });
        }
    }
}

fn detect_macd_divergence(klines: &[KlineItem], indicators: &TechnicalIndicators, signals: &mut Vec<TechnicalSignal>) {
    let n = klines.len();
    if n < 30 { return; }

    let check_range = n.saturating_sub(30)..n;

    // 找最近30日内的两个价格高点和对应的MACD DIF
    let mut highs_points: Vec<(usize, f64, f64)> = Vec::new(); // (index, price_high, dif)
    for i in check_range.clone() {
        if i < 1 || i >= n - 1 { continue; }
        if klines[i].high > klines[i-1].high && klines[i].high > klines[i.min(n-1)].high {
            if let Some(dif) = indicators.macd_dif[i] {
                highs_points.push((i, klines[i].high, dif));
            }
        }
    }

    // 顶背离：价格新高但MACD未新高
    if highs_points.len() >= 2 {
        let last = highs_points.last().unwrap();
        let prev = &highs_points[highs_points.len() - 2];
        if last.1 > prev.1 && last.2 < prev.2 {
            signals.push(TechnicalSignal {
                signal_type: "macd_top_divergence".into(),
                direction: "bearish".into(),
                description: "MACD顶背离".into(),
                strength: 5,
                date: klines[last.0].date.clone(),
            });
        }
    }

    // 找底部低点
    let mut lows_points: Vec<(usize, f64, f64)> = Vec::new();
    for i in check_range {
        if i < 1 || i >= n - 1 { continue; }
        if klines[i].low < klines[i-1].low && klines[i].low < klines[i.min(n-1)].low {
            if let Some(dif) = indicators.macd_dif[i] {
                lows_points.push((i, klines[i].low, dif));
            }
        }
    }

    // 底背离：价格新低但MACD未新低
    if lows_points.len() >= 2 {
        let last = lows_points.last().unwrap();
        let prev = &lows_points[lows_points.len() - 2];
        if last.1 < prev.1 && last.2 > prev.2 {
            signals.push(TechnicalSignal {
                signal_type: "macd_bottom_divergence".into(),
                direction: "bullish".into(),
                description: "MACD底背离".into(),
                strength: 5,
                date: klines[last.0].date.clone(),
            });
        }
    }
}
