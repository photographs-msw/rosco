Prequisite, follow the instructions for your platform in `tools/install*.sh` to install `liblo`

### Create an Oscillator Note TODO AND QUEUE IT?

**Arguments:**
- `frequency` (f32): Frequency in Hz (e.g., 440.0 for A4)
- `volume` (f32): Volume level 0.0-1.0
- `start_time_ms` (f32): Start time in milliseconds
- `duration_ms` (f32): Duration in milliseconds
- `waveforms` (string): Comma-separated list of waveforms (sine, square, triangle, saw, noise)

```bash
# liblo uses UDP by default, this server serves UDP on 8081 by default
> oscsend localhost 8001 /note/oscillator ffffs 440.0 0.5 0.0 1000.0 "sine"
```

### Create a Sampled Note and Queue it

**Arguments:**
- `file_path` (string): Path to the audio file
- `volume` (f32): Volume level 0.0-1.0
- `start_time_ms` (f32): Start time in milliseconds
- `duration_ms` (f32): Duration in milliseconds

```bash
# liblo uses UDP by default, this server serves UDP on 8081 by default
> oscsend localhost 8001 /note/sample sfff "/path/to/file.wav" 0.5 0.0 1000.0
```

