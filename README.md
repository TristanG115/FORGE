# FORGE

**F**orm **O**riented **R**eference **G**eneration **E**ngine

FORGE is a deterministic, local-first asset generation tool designed for game development. It provides an AI-assisted (parameter-only) pipeline that transforms 2D silhouettes and images into controlled 3D game assets, running entirely on local machines without any cloud services.

## Overview

FORGE employs a modular, Rust architecture to deliver a deterministic 2D-to-3D asset pipeline with the following key features:

- **Deterministic Processing**: Reproducible results through parameter-based control
- **AI-Assisted Generation**: Parameter-only AI integration for predictable outputs
- **Session Management**: Persistent workflow sessions with full state tracking
- **Local Execution**: All processing runs on local machines - no cloud dependencies
- **Unreal Engine Export**: Native support for Unreal Engine asset formats
- **Modular Design**: Clean separation of concerns across specialized modules

## Architecture

FORGE is organized into four core modules:

### forge-core
Rust-based deterministic asset processing engine providing fundamental pipeline operations:
- Asset data structures and management
- Deterministic transformation algorithms
- Session state management
- File I/O and serialization
- Export format handlers (Unreal Engine, etc.)

### forge-variation
Variation generation module for creating controlled asset variations:
- Parameter-based variation generation
- 2D reference image manipulation
- Deterministic variation algorithms
- Variation preview and approval workflow

### forge-ai
AI integration module for the asset pipeline:
- AI model parameter management (parameter-only control)
- 2D to 3D asset generation
- Local model execution
- Model inference pipeline
- No cloud dependencies - all processing runs locally

### forge-ui
User interface module for asset creation and management:
- Asset preview and review interface
- Parameter adjustment controls
- Session management UI
- Export configuration
- Variation approval workflow

## Asset Pipeline

The FORGE pipeline follows a deterministic, multi-stage process:

1. **Input**: Load 2D silhouettes, sketches, or reference images
2. **Variation**: Generate controlled 2D variations using deterministic parameters
3. **Review**: User reviews and approves variations before 3D generation
4. **Generation**: AI-assisted conversion to 3D assets (parameter-only control)
5. **Export**: Output to Unreal Engine or other supported formats

All stages maintain session state, allowing users to pause, resume, and iterate on their work.

## Key Features

- **Deterministic Output**: Same inputs and parameters always produce identical results
- **Parameter-Only AI**: AI models controlled exclusively through parameters, ensuring reproducibility
- **Session Persistence**: Full session state saves enable pause/resume workflows
- **Local Processing**: Zero cloud dependencies - all computation runs on user's machine
- **Modular C Core**: High-performance C implementation with clean module boundaries
- **Unreal Integration**: Direct export to Unreal Engine asset formats

## Getting Started

*Documentation for building and running FORGE will be added as modules are implemented.*

## License

FORGE is licensed under the Elastic License 2.0. See [LICENSE](LICENSE) for the full license text.
