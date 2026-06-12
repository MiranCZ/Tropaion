use std::collections::HashMap;

pub struct ProfilingData {
    pub total_cost: u64,
    pub insn_count: u64,
    pub by_instruction: HashMap<String, u64>,
    pub by_line: HashMap<usize, u64>,
    // insn_addr -> (exec_count, total_cost)
    pub insn_stats: HashMap<usize, (u64, u64)>,
}

impl ProfilingData {
    pub fn new() -> Self {
        Self {
            total_cost: 0,
            insn_count: 0,
            by_instruction: HashMap::new(),
            by_line: HashMap::new(),
            insn_stats: HashMap::new(),
        }
    }

    pub fn record(&mut self, insn_addr: usize, insn_name: &str, line: usize, cost: u64) {
        self.total_cost += cost;
        self.insn_count += 1;
        *self.by_instruction.entry(insn_name.to_string()).or_insert(0) += cost;
        *self.by_line.entry(line).or_insert(0) += cost;
        let entry = self.insn_stats.entry(insn_addr).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += cost;
    }

    // insn_reprs: format!("{:?}") for each bytecode in program order
    // line_maps:  source line for each bytecode position (parallel to insn_reprs)
    pub fn generate_html(&self, source: &str, wall_ms: u128, insn_reprs: &[String], line_maps: &[usize]) -> String {
        let total = self.total_cost.max(1);

        // --- global instruction breakdown table ---
        let mut insns: Vec<(&str, u64)> = self
            .by_instruction
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect();
        insns.sort_by(|a, b| b.1.cmp(&a.1));

        let mut insn_rows = String::new();
        for (name, cost) in &insns {
            let pct = *cost as f64 / total as f64 * 100.0;
            let bar_w = pct.min(100.0) as u32;
            insn_rows.push_str(&format!(
                "<tr>\
<td class=\"name\">{name}</td>\
<td class=\"num\">{cost}</td>\
<td class=\"num\">{pct:.1}%</td>\
<td class=\"bar-cell\"><div class=\"bar\" style=\"width:{bar_w}%\"></div></td>\
</tr>"
            ));
        }

        // --- raw bytecode table (all instructions in program order) ---
        let mut raw_rows = String::new();
        for (addr, repr) in insn_reprs.iter().enumerate() {
            let line = line_maps.get(addr).copied().unwrap_or(0);
            let escaped_repr = escape_html(repr);
            match self.insn_stats.get(&addr) {
                None => {
                    raw_rows.push_str(&format!(
                        "<tr class=\"bi-dead\"><td class=\"num\">{addr}</td><td class=\"num\">{line}</td><td class=\"name\">{escaped_repr}</td><td></td><td></td><td></td></tr>\n"
                    ));
                }
                Some((cnt, icost)) => {
                    let ipct = *icost as f64 / total as f64 * 100.0;
                    raw_rows.push_str(&format!(
                        "<tr><td class=\"num\">{addr}</td><td class=\"num\">{line}</td><td class=\"name\">{escaped_repr}</td><td class=\"num\">×{cnt}</td><td class=\"num\">{icost}</td><td class=\"num\">{ipct:.2}%</td></tr>\n"
                    ));
                }
            }
        }

        // pre-group static bytecode addresses by source line (preserving program order)
        let mut addrs_by_line: HashMap<usize, Vec<usize>> = HashMap::new();
        for (addr, &line) in line_maps.iter().enumerate() {
            addrs_by_line.entry(line).or_default().push(addr);
        }

        // --- unmapped (line 0) instructions row ---
        let unmapped_row = match addrs_by_line.get(&0) {
            None => String::new(),
            Some(addrs) => {
                let mut inner = String::new();
                for addr in addrs {
                    let repr = insn_reprs.get(*addr).map(|s| s.as_str()).unwrap_or("?");
                    let escaped_repr = escape_html(repr);
                    match self.insn_stats.get(addr) {
                        None => {
                            inner.push_str(&format!(
                                "<div class=\"bi bi-dead\"><span class=\"bi-name\">{escaped_repr}</span></div>"
                            ));
                        }
                        Some((cnt, icost)) => {
                            let ipct = *icost as f64 / total as f64 * 100.0;
                            inner.push_str(&format!(
                                "<div class=\"bi\">\
<span class=\"bi-name\">{escaped_repr}</span>\
<span class=\"bi-x\">×{cnt}</span>\
<span class=\"bi-cost\">{icost} ({ipct:.2}%)</span>\
</div>"
                            ));
                        }
                    }
                }
                format!(
                    "<tr class=\"unmapped-row\">\
<td class=\"src-col\" colspan=\"2\">\
<details><summary><span class=\"unmapped-label\">compiler-generated ({} instructions, no source line)</span></summary>\
<div class=\"unmapped-bc\">{inner}</div></details>\
</td></tr>\n",
                    addrs.len()
                )
            }
        };

        // --- source + bytecode side-by-side rows ---
        let max_line_cost = self.by_line.values().copied().max().unwrap_or(1).max(1);
        let lines: Vec<&str> = source.lines().collect();
        let mut table_rows = String::new();

        for (i, line_text) in lines.iter().enumerate() {
            let line_no = i + 1;
            let cost = self.by_line.get(&line_no).copied().unwrap_or(0);
            let pct = cost as f64 / total as f64 * 100.0;

            let (cost_str, color_style) = if cost == 0 {
                (String::from("        "), String::from("color:#555"))
            } else {
                let fraction = cost as f64 / max_line_cost as f64;
                let hue = 120.0 - fraction * 120.0;
                let lightness = 60.0 - fraction * 15.0;
                (
                    format!("{pct:>7.1}%"),
                    format!("color:hsl({hue:.0},80%,{lightness:.0}%)"),
                )
            };

            let escaped_line = escape_html(line_text);
            let src_cell = format!(
                "<span class=\"ln\">{line_no:>4}</span> <span class=\"lc\">{cost_str}</span> {escaped_line}"
            );

            let bc_cell = match addrs_by_line.get(&line_no) {
                None => String::new(),
                Some(addrs) => {
                    let mut inner = String::new();
                    for addr in addrs {
                        let repr = insn_reprs.get(*addr).map(|s| s.as_str()).unwrap_or("?");
                        let escaped_repr = escape_html(repr);
                        match self.insn_stats.get(addr) {
                            None => {
                                inner.push_str(&format!(
                                    "<div class=\"bi bi-dead\"><span class=\"bi-name\">{escaped_repr}</span></div>"
                                ));
                            }
                            Some((cnt, icost)) => {
                                let ipct = *icost as f64 / total as f64 * 100.0;
                                inner.push_str(&format!(
                                    "<div class=\"bi\">\
<span class=\"bi-name\">{escaped_repr}</span>\
<span class=\"bi-x\">×{cnt}</span>\
<span class=\"bi-cost\">{icost} ({ipct:.2}%)</span>\
</div>"
                                ));
                            }
                        }
                    }
                    inner
                }
            };

            table_rows.push_str(&format!(
                "<tr style=\"{color_style}\">\
<td class=\"src-col\">{src_cell}</td>\
<td class=\"bc-col\">{bc_cell}</td>\
</tr>\n"
            ));
        }

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Tropaion Profile</title>
<style>
* {{ box-sizing: border-box; margin: 0; padding: 0; }}
body {{ background: #1a1a2e; color: #e0e0e0; font-family: 'Segoe UI', sans-serif; padding: 24px; }}
h1 {{ font-size: 1.6rem; margin-bottom: 16px; color: #a0c4ff; }}
h2 {{ font-size: 1.1rem; margin: 24px 0 10px; color: #a0c4ff; border-bottom: 1px solid #333; padding-bottom: 4px; }}
.summary {{ display: flex; gap: 20px; flex-wrap: wrap; margin-bottom: 8px; }}
.stat {{ background: #16213e; border-radius: 8px; padding: 14px 22px; }}
.stat-label {{ font-size: 0.75rem; color: #888; text-transform: uppercase; letter-spacing: .05em; }}
.stat-value {{ font-size: 1.5rem; font-weight: 700; color: #e0e0e0; margin-top: 2px; }}
.insn-table {{ border-collapse: collapse; width: 100%; font-size: 0.85rem; }}
.insn-table th {{ text-align: left; color: #888; font-weight: 600; padding: 6px 10px; border-bottom: 1px solid #333; }}
.insn-table td {{ padding: 5px 10px; border-bottom: 1px solid #222; vertical-align: middle; }}
td.name {{ font-family: monospace; color: #ffd6a5; }}
td.num {{ text-align: right; font-family: monospace; color: #ccc; }}
td.bar-cell {{ width: 200px; }}
.bar {{ height: 10px; background: #4cc9f0; border-radius: 3px; min-width: 2px; }}
.toolbar {{ margin: 12px 0 8px; }}
.toggle-btn {{
  background: #16213e; border: 1px solid #2a3a5a; color: #a0c4ff;
  padding: 6px 16px; border-radius: 6px; cursor: pointer; font-size: 0.85rem;
}}
.toggle-btn:hover {{ background: #1e2f5a; }}
.hidden {{ display: none; }}
#raw-wrap {{ margin-bottom: 8px; }}
.source-wrap {{ background: #0f0f1a; border-radius: 8px; padding: 0; overflow-x: auto; }}
.src-table {{ border-collapse: collapse; width: 100%; font-family: monospace; font-size: 0.82rem; }}
.src-table tr {{ line-height: 1.7; }}
.src-table tr:hover {{ background: #141428; }}
.src-col {{ white-space: pre; padding: 0 16px; vertical-align: top; }}
.bc-col {{
  border-left: 1px solid #1e2a3a; padding: 2px 12px 2px 16px;
  vertical-align: top; min-width: 360px;
}}
.bc-col.hidden {{ display: none; }}
.ln {{ color: #555; user-select: none; }}
.lc {{ font-variant-numeric: tabular-nums; user-select: none; }}
.bi {{ display: flex; gap: 10px; padding: 1px 0; white-space: nowrap; color: #aaa; }}
.bi-dead {{ opacity: 0.35; }}
.bi-name {{ color: #ffd6a5; min-width: 24ch; }}
.bi-x {{ color: #666; min-width: 6ch; }}
.bi-cost {{ color: #a0c4ff; }}
.unmapped-row td {{ background: #0d0d1a; border-bottom: 1px solid #2a2a4a; }}
.unmapped-label {{ color: #556; font-style: italic; font-size: 0.8rem; cursor: pointer; padding: 4px 16px; display: block; }}
.unmapped-label:hover {{ color: #778; }}
details > summary {{ list-style: none; }}
details > summary::-webkit-details-marker {{ display: none; }}
.unmapped-bc {{ padding: 4px 32px 8px; }}
</style>
</head>
<body>
<h1>Tropaion Execution Profile</h1>

<div class="summary">
  <div class="stat"><div class="stat-label">Total cost</div><div class="stat-value">{total}</div></div>
  <div class="stat"><div class="stat-label">Instructions executed</div><div class="stat-value">{insn_count}</div></div>
  <div class="stat"><div class="stat-label">Wall time</div><div class="stat-value">{wall_ms} ms</div></div>
</div>

<h2>Instruction Breakdown</h2>
<table class="insn-table">
  <thead><tr><th>Instruction</th><th>Cost</th><th>%</th><th style="width:200px"></th></tr></thead>
  <tbody>{insn_rows}</tbody>
</table>

<h2>Raw Bytecode</h2>
<div class="toolbar">
  <button class="toggle-btn" onclick="toggleRaw(this)">Show Raw</button>
</div>
<div id="raw-wrap" class="hidden">
  <table class="insn-table">
    <thead><tr><th>Addr</th><th>Line</th><th>Instruction</th><th>×</th><th>Cost</th><th>%</th></tr></thead>
    <tbody>{raw_rows}</tbody>
  </table>
</div>

<h2>Source View</h2>
<div class="toolbar">
  <button class="toggle-btn" onclick="toggleBytecode(this)">Show Bytecode</button>
</div>
<div class="source-wrap">
  <table class="src-table">
    <tbody>{unmapped_row}{table_rows}</tbody>
  </table>
</div>

<script>
function toggleRaw(btn) {{
  const el = document.getElementById('raw-wrap');
  const hidden = el.classList.toggle('hidden');
  btn.textContent = hidden ? 'Show Raw' : 'Hide Raw';
}}
function toggleBytecode(btn) {{
  const cols = document.querySelectorAll('.bc-col');
  const nowHidden = cols[0].classList.contains('hidden');
  cols.forEach(el => el.classList.toggle('hidden', !nowHidden));
  btn.textContent = nowHidden ? 'Hide Bytecode' : 'Show Bytecode';
}}
document.querySelectorAll('.bc-col').forEach(el => el.classList.add('hidden'));
</script>
</body>
</html>"#,
            total = self.total_cost,
            insn_count = self.insn_count,
        )
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
