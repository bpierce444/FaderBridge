# Performance Metrics: FaderBridge

**Version:** 0.1.0  
**Last Updated:** [DATE]  

---

## Target Metrics (From PROJECT_CHARTER.md)

| Metric | Target | Priority |
|--------|--------|----------|
| MIDI→UCNet Latency | < 10ms | Critical |
| UCNet→MIDI Latency | < 10ms | Critical |
| App Launch Time | < 2 seconds | High |
| Memory Usage (1 hour) | < 150 MB | High |
| CPU Usage (normal) | < 10% | Medium |
| Stability | 4 hours no crash | Critical |

---

## Latency Metrics

### MIDI to UCNet Translation

The sync engine measures latency from MIDI message receipt to UCNet parameter update.

#### Measurement Points
1. **Start:** `Instant::now()` when MIDI message enters `handle_midi_message()`
2. **End:** After shadow state update and before UCNet send

#### Current Results
| Test Date | Samples | Avg (ms) | Min (ms) | Max (ms) | P95 (ms) | Status |
|-----------|---------|----------|----------|----------|----------|--------|
| [date] | [n] | [avg] | [min] | [max] | [p95] | [PASS/FAIL] |

#### Latency Breakdown
| Stage | Typical Time | Notes |
|-------|--------------|-------|
| MIDI parsing | < 0.1ms | Negligible |
| Mapping lookup | < 0.1ms | HashMap O(1) |
| Value translation | < 0.1ms | Math operations |
| Shadow state check | < 0.5ms | RwLock contention |
| UCNet serialization | < 0.5ms | JSON encoding |
| **Total Internal** | < 1.5ms | Before network |
| Network transmission | 1-5ms | Variable |
| **Total End-to-End** | < 10ms | Target |

### UCNet to MIDI Translation

#### Current Results
| Test Date | Samples | Avg (ms) | Min (ms) | Max (ms) | P95 (ms) | Status |
|-----------|---------|----------|----------|----------|----------|--------|
| [date] | [n] | [avg] | [min] | [max] | [p95] | [PASS/FAIL] |

---

## Memory Metrics

### Memory Usage Over Time

| Time | Target | Measured | Delta | Status |
|------|--------|----------|-------|--------|
| Launch | < 100 MB | [MB] | - | [PASS/FAIL] |
| 15 min | < 120 MB | [MB] | [+MB] | [PASS/FAIL] |
| 30 min | < 130 MB | [MB] | [+MB] | [PASS/FAIL] |
| 1 hour | < 150 MB | [MB] | [+MB] | [PASS/FAIL] |
| 2 hours | < 150 MB | [MB] | [+MB] | [PASS/FAIL] |
| 4 hours | < 150 MB | [MB] | [+MB] | [PASS/FAIL] |

### Memory Breakdown (Estimated)
| Component | Estimated Size | Notes |
|-----------|----------------|-------|
| Tauri Runtime | ~50 MB | Base overhead |
| WebView | ~30 MB | React UI |
| Rust Backend | ~10 MB | Protocol logic |
| Shadow State | ~1 MB per 1000 params | Scales with mappings |
| Latency Samples | ~8 KB per 1000 samples | Fixed buffer |

### Memory Leak Indicators
- [ ] Memory grows unbounded over time
- [ ] Memory not released after clearing mappings
- [ ] Memory not released after device disconnect

---

## CPU Metrics

### CPU Usage by State

| State | Target | Measured | Status |
|-------|--------|----------|--------|
| Idle (no activity) | < 5% | [%] | [PASS/FAIL] |
| Normal operation | < 10% | [%] | [PASS/FAIL] |
| Heavy sync (100+ msg/sec) | < 25% | [%] | [PASS/FAIL] |
| Project save | < 50% (spike) | [%] | [PASS/FAIL] |

### CPU Breakdown (Estimated)
| Component | Typical Usage | Notes |
|-----------|---------------|-------|
| MIDI polling | < 1% | Event-driven |
| UCNet keep-alive | < 1% | Periodic |
| UI rendering | 1-5% | React updates |
| Sync engine | < 2% | Per message |

---

## Startup Metrics

### Launch Time Breakdown

| Stage | Target | Measured | Status |
|-------|--------|----------|--------|
| App binary load | < 500ms | [ms] | [PASS/FAIL] |
| Tauri initialization | < 500ms | [ms] | [PASS/FAIL] |
| WebView load | < 500ms | [ms] | [PASS/FAIL] |
| React hydration | < 300ms | [ms] | [PASS/FAIL] |
| Device discovery start | < 200ms | [ms] | [PASS/FAIL] |
| **Total to Interactive** | < 2000ms | [ms] | [PASS/FAIL] |

---

## Throughput Metrics

### Message Processing Rate

| Direction | Target | Measured | Status |
|-----------|--------|----------|--------|
| MIDI messages/sec | > 1000 | [n] | [PASS/FAIL] |
| UCNet messages/sec | > 100 | [n] | [PASS/FAIL] |
| Bidirectional | > 500 | [n] | [PASS/FAIL] |

### Stress Test Results
| Test | Duration | Messages | Errors | Status |
|------|----------|----------|--------|--------|
| Rapid fader sweep | 60 sec | [n] | [n] | [PASS/FAIL] |
| 8 simultaneous faders | 60 sec | [n] | [n] | [PASS/FAIL] |
| Mute spam | 60 sec | [n] | [n] | [PASS/FAIL] |

---

## UI Performance Metrics

### Frame Rate
| Scenario | Target | Measured | Status |
|----------|--------|----------|--------|
| Idle | 60 fps | [fps] | [PASS/FAIL] |
| Fader animation | 60 fps | [fps] | [PASS/FAIL] |
| List scrolling | 60 fps | [fps] | [PASS/FAIL] |

### Interaction Latency
| Action | Target | Measured | Status |
|--------|--------|----------|--------|
| Button click | < 100ms | [ms] | [PASS/FAIL] |
| Dropdown open | < 100ms | [ms] | [PASS/FAIL] |
| Modal open | < 200ms | [ms] | [PASS/FAIL] |
| Project load | < 1000ms | [ms] | [PASS/FAIL] |

---

## Profiling Tools

### macOS Activity Monitor
- **Memory:** Real Memory column
- **CPU:** % CPU column
- **Energy:** Energy Impact column

### Instruments.app
- **Time Profiler:** CPU hotspots
- **Allocations:** Memory allocations
- **Leaks:** Memory leak detection
- **Network:** UCNet traffic analysis

### Built-in Metrics
The app exposes latency metrics via the Status Bar:
- Average latency (last 1000 samples)
- Min/Max latency
- Sample count

Access via Tauri command:
```typescript
const stats = await invoke('get_latency_stats');
// Returns: { avg_ms, min_ms, max_ms, sample_count }
```

---

## Optimization History

### Version 0.1.0
| Date | Change | Impact |
|------|--------|--------|
| [date] | Initial implementation | Baseline |

---

## Benchmarking Procedure

### Latency Benchmark
```bash
# 1. Start the app with logging enabled
RUST_LOG=debug cargo tauri dev

# 2. Create a test mapping
# 3. Use a MIDI controller to send 1000 CC messages
# 4. Collect latency samples from logs
# 5. Calculate statistics
```

### Memory Benchmark
```bash
# 1. Start the app
# 2. Record initial memory from Activity Monitor
# 3. Run test scenario for 4 hours
# 4. Record memory at intervals
# 5. Check for growth trend
```

### CPU Benchmark
```bash
# 1. Start the app
# 2. Use Instruments Time Profiler
# 3. Run test scenario
# 4. Analyze CPU usage by function
```

---

## Performance Regression Tests

### Automated Checks (CI)
- [ ] Unit test latency assertions (< 10ms in tests)
- [ ] Memory allocation tests (no unbounded growth)

### Manual Checks (Pre-Release)
- [ ] 4-hour stability test
- [ ] Memory profiling with Instruments
- [ ] CPU profiling with Instruments
- [ ] Real hardware latency measurement

---

## Appendix: Raw Benchmark Data

[Link to or embed raw benchmark data files]
