# MSCI Security Crowding（公开数据近似）

依据 [MSCI Security Crowding Model Factsheet](https://www.msci.com/documents/1296102/25918384/MSCI-Security-Crowding-Model-Factsheet.pdf) 的层级与权重，用 **yfinance** 行情/基本面做可复现近似，**不是** MSCI 商业数据源。

## 公式层级

```
Integrated = 0.5 * TimeSeries + 0.5 * CrossSection

TimeSeries / CrossSection =
  0.40 * Value
+ 0.30 * ShortInterest
+ 0.10 * Liquidity(Turnover)
+ 0.10 * ResidualVolatility
+ 0.10 * Momentum

Value = 0.5 * EarningsYield + 0.5 * BookToPrice
      （拥挤侧取昂贵：对 EY/BP 取负后再做 z-score）
```

每个叶子指标先算 **z-score**：

- **Time-Series**：相对自身约 3 年日频历史  
- **Cross-Sectional**：相对当期同业样本（默认更大市值池，避免仅 7 只导致 XS 失真）

解读（MSCI 公开说明）：分数约一半 >0；**>2 非常拥挤**；本脚本将 **≥1** 标为 CROWDED。

## 市值与日期

- **价格日期**：yfinance 最近交易日收盘（结果表 `as_of` 列）
- **真实市值**：`last_close × impliedSharesOutstanding`（双层股权用 implied，避免只算单一 share class）
- **排名宇宙**：从 [companiesmarketcap US](https://companiesmarketcap.com/usa/largest-companies-in-the-usa-by-market-cap/) 拉取候选，再按上述市值重排

## 运行

```bash
pip install -r scripts/crowding/requirements.txt
python scripts/crowding/run_top7.py --top 20 --universe 40
```

## 数据近似说明

| 组件 | 公开近似 |
|------|----------|
| Valuation | trailing PE / PB → EY、BP，再取负表示昂贵 |
| Short Interest | `shortPercentOfFloat`（历史序列弱，TS 常接近中性） |
| Liquidity | 20 日均换手 ≈ volume / sharesOutstanding |
| Residual Vol | 对 SPY 回归残差的滚动年化波动 |
| Momentum | 12–1 月动量 |

PDF 副本：`MSCI-Security-Crowding-Model-Factsheet.pdf`（同目录）。
