# Contributing to CX Framework

Thank you for your interest in contributing to CX Framework! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contribution Guidelines](#contribution-guidelines)
- [Testing](#testing)
- [Documentation](#documentation)
- [Submitting Changes](#submitting-changes)
- [Release Process](#release-process)

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md). Please read it before contributing.

## Getting Started

### Areas for Contribution

We welcome contributions in these areas:

- 🐛 **Bug Fixes**: Fix issues and improve stability
- ✨ **New Features**: Add widgets, layouts, or framework capabilities
- 📚 **Documentation**: Improve docs, guides, and examples
- 🧪 **Testing**: Add tests and improve test coverage
- 🚀 **Performance**: Optimize rendering and memory usage
- 🔧 **Platform Support**: Improve cross-platform compatibility

### Good First Issues

Look for issues tagged with:
- `good-first-issue`: Perfect for newcomers
- `help-wanted`: Community help needed
- `documentation`: Documentation improvements
- `easy`: Low complexity changes

## Development Setup

### Prerequisites

Ensure you have the following installed:

- **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
- **Platform Tools**:
  - **Windows**: Visual Studio Build Tools
  - **Linux**: X11 development headers (`libx11-dev`)
  - **macOS**: Xcode Command Line Tools

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/cx-framework/cx-framework.git
cd cx-framework

# Build the project
cargo build

# Run tests
cargo test

# Build documentation
cargo doc --open
```

### Development Dependencies

For development, install additional tools:

```bash
# Code formatting
rustup component add rustfmt

# Linting
rustup component add clippy

# Code coverage (optional)
cargo install cargo-tarpaulin
```

## Contribution Guidelines

### Code Style

We follow standard Rust conventions:

1. **Formatting**: Use `cargo fmt` before committing
2. **Linting**: Address all `cargo clippy` warnings
3. **Naming**: Use descriptive names following Rust conventions
4. **Comments**: Document public APIs and complex logic

Example:

```rust
/// Creates a new button widget with the specified text.
/// 
/// # Arguments
/// 
/// * `id` - Unique identifier for the widget
/// * `text` - Display text for the button
/// 
/// # Examples
/// 
/// ```
/// let button = Button::new(1, "Click me!".to_string());
/// ```
pub fn new(id: WidgetId, text: String) -> Self {
    Self {
        base: BaseWidget::new(id),
        text,
        // ...
    }
}
```

### Architecture Guidelines

#### Platform Abstraction

Keep platform-specific code isolated:

```rust
// ✅ Good: Platform-specific code in backend
#[cfg(windows)]
mod windows {
    // Windows-specific implementation
}

#[cfg(unix)]
mod unix {
    // Unix-specific implementation
}

// ✅ Good: Abstract interface
pub trait PlatformBackend {
    fn create_window(&mut self, params: &WindowParams) -> Result<WindowHandle>;
}
```

```rust
// ❌ Bad: Platform-specific code in public API
pub fn create_window_win32(hwnd: HWND) -> Window {
    // Don't expose platform details
}
```

#### Error Handling

Use the framework's error types consistently:

```rust
// ✅ Good: Consistent error handling
pub fn create_widget(&self) -> Result<Widget> {
    let widget = Widget::new()
        .map_err(|e| Error::widget("Failed to create widget"))?;
    Ok(widget)
}

// ❌ Bad: Inconsistent error types
pub fn create_widget(&self) -> std::io::Result<Widget> {
    // Don't mix error types
}
```

### Widget Development

When creating new widgets:

1. **Inherit from BaseWidget**: Use `BaseWidget` for common functionality
2. **Implement Widget Trait**: All widgets must implement the `Widget` trait
3. **Add Builder Pattern**: Support fluent configuration
4. **Handle Events**: Implement appropriate event handling
5. **Support Styling**: Allow customization of appearance

Example widget structure:

```rust
pub struct CustomWidget {
    base: BaseWidget,
    custom_property: String,
}

impl CustomWidget {
    pub fn new(id: WidgetId) -> Self {
        Self {
            base: BaseWidget::new(id),
            custom_property: String::new(),
        }
    }
    
    /// Builder method for custom property
    pub fn custom_property<S: Into<String>>(mut self, value: S) -> Self {
        self.custom_property = value.into();
        self
    }
}

impl Widget for CustomWidget {
    // Implement required methods
    fn render(&self, canvas: &mut dyn Canvas) -> Result<()> {
        // Custom rendering logic
    }
    
    fn handle_event(&mut self, event: &Event) -> Result<bool> {
        // Custom event handling
        Ok(false)
    }
    
    // ... other required methods
}
```

## Testing

### Test Categories

We maintain several types of tests:

#### Unit Tests
Test individual components in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_widget_creation() {
        let widget = CustomWidget::new(1);
        assert_eq!(widget.id(), 1);
    }
}
```

#### Integration Tests
Test component interactions:

```rust
// tests/integration_test.rs
#[test]
fn test_window_with_widgets() -> Result<()> {
    let app = Application::new()?;
    let window = WindowBuilder::new().build(&app)?;
    
    let button = Button::new(1, "Test".to_string());
    // Test widget integration
    
    Ok(())
}
```

#### Platform Tests
Test platform-specific functionality:

```rust
#[cfg(windows)]
#[test]
fn test_windows_specific_feature() {
    // Windows-only test
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_widget_creation

# Run platform-specific tests
cargo test --features windows-tests

# Generate coverage report
cargo tarpaulin --out Html
```

### Test Guidelines

1. **Comprehensive Coverage**: Aim for high test coverage
2. **Platform Testing**: Test on all supported platforms
3. **Error Cases**: Test both success and failure scenarios
4. **Performance**: Include performance regression tests
5. **Documentation Tests**: Ensure examples in docs compile

## Documentation

### API Documentation

All public APIs must be documented:

```rust
/// Widget that displays clickable text.
/// 
/// Buttons can be styled with custom colors, fonts, and sizes.
/// They emit click events when activated by mouse or keyboard.
/// 
/// # Examples
/// 
/// ```
/// use cx::Button;
/// 
/// let button = Button::new(1, "Click me!".to_string())
///     .background_color(Color::BLUE)
///     .text_color(Color::WHITE);
/// ```
pub struct Button {
    // ...
}
```

### User Guides

When adding major features:

1. Update the [Getting Started Guide](docs/GETTING_STARTED.md)
2. Add example code to the [examples](examples/) directory
3. Update the main [README.md](README.md) if needed

### Documentation Standards

- Use complete sentences
- Include examples for public APIs
- Explain the "why" not just the "what"
- Link to related concepts
- Keep examples simple but realistic

## Submitting Changes

### Pull Request Process

1. **Fork the Repository**: Create your own fork on GitHub

2. **Create a Branch**: Use descriptive branch names
   ```bash
   git checkout -b feature/new-widget
   git checkout -b fix/memory-leak
   git checkout -b docs/update-readme
   ```

3. **Make Changes**: Follow the guidelines above

4. **Test Your Changes**: Ensure all tests pass
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

5. **Commit Changes**: Use clear commit messages
   ```bash
   git commit -m "Add TreeView widget with expand/collapse support"
   git commit -m "Fix memory leak in window cleanup"
   git commit -m "Update documentation for Button widget"
   ```

6. **Submit Pull Request**: Create a PR with:
   - Clear title describing the change
   - Detailed description of what and why
   - Reference to related issues
   - Screenshots for UI changes
   - Test results on different platforms

### Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Other (please describe)

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed
- [ ] Tested on Windows
- [ ] Tested on Linux
- [ ] Tested on macOS

## Screenshots (if applicable)
[Add screenshots for UI changes]

## Related Issues
Fixes #123
Closes #456
```

### Review Process

1. **Automated Checks**: CI will run tests and linting
2. **Code Review**: Maintainers will review your code
3. **Feedback**: Address any requested changes
4. **Approval**: Once approved, your PR will be merged

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR.MINOR.PATCH** (e.g., 1.2.3)
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Schedule

- **Patch releases**: As needed for critical fixes
- **Minor releases**: Monthly for new features
- **Major releases**: Every 6-12 months for breaking changes

### Release Checklist

For maintainers releasing new versions:

1. Update `CHANGELOG.md`
2. Update version in `Cargo.toml`
3. Run full test suite on all platforms
4. Build and test examples
5. Generate documentation
6. Create release tag
7. Publish to crates.io
8. Update GitHub release notes

## Community

### Getting Help

- **GitHub Issues**: Report bugs and request features
- **GitHub Discussions**: Ask questions and share ideas
- **Discord**: Real-time chat with the community
- **Stack Overflow**: Tag questions with `cx-framework`

### Recognition

Contributors are recognized:

- Listed in `CONTRIBUTORS.md`
- Mentioned in release notes
- Invited to maintainer status (for significant contributions)

## License

By contributing to CX Framework, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to CX Framework! Your efforts help make cross-platform GUI development in Rust better for everyone.