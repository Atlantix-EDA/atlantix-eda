# Contributing to Atlantix-EDA

Thank you for your interest in contributing to Atlantix-EDA! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and collaborative environment. We expect all contributors to:

- Be cognizant of different levels of technical expertise
- Focus on technical merit and constructive feedback
- Only say what you would say in person, face to face!
- Welcome newcomers and help them get started

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR-USERNAME/atlantix-eda.git
   cd atlantix-eda
   ```
3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/saturn77/atlantix-eda.git
   ```
4. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Guidelines

### Code Style

- Follow Rust's official style guidelines (use `rustfmt`)
- Write clear, self-documenting code
- Try to follow the existing code structure and patterns
- Use meaningful variable and function names
- Keep functions focused and small

### Commit Messages

- Use clear, descriptive commit messages
- Start with a verb in present tense (e.g., "Add", "Fix", "Update", "Refactor")
- Keep the first line under 50 characters
- Add detailed description after a blank line if needed

Example:
```
Add KiCad symbol generation for capacitors

- Implement capacitor symbol generation following IEC standards
- Support multiple package sizes (0402, 0603, 0805)
- Add unit tests for symbol validation
```

### Testing

- Write tests for new functionality
- Ensure all existing tests pass before submitting PR:
  ```bash
  cargo test
  ```
- Test your changes with real-world use cases
- Include integration tests for new features

### Documentation

- Update relevant documentation for any API changes
- Add rustdoc comments for public functions and modules
- Update README.md if adding new features or changing usage
- Include examples for complex functionality

## Pull Request Process

1. **Update your fork** with latest upstream changes:
   ```bash
   git fetch upstream
   git checkout master
   git merge upstream/master
   ```

2. **Rebase your feature branch**:
   ```bash
   git checkout feature/your-feature-name
   git rebase master
   ```

3. **Run quality checks**:
   ```bash
   cargo fmt -- --check
   cargo clippy -- -D warnings
   cargo test
   cargo build --release
   ```

4. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

5. **Create Pull Request** on GitHub:
   - Provide clear description of changes
   - Reference any related issues
   - Include screenshots/examples if applicable
   - Explain testing performed

### PR Requirements

- **One feature per PR** - Keep PRs focused and manageable
- **All tests must pass** - Including CI/CD checks
- **No breaking changes** without discussion
- **Maintain backward compatibility** when possible
- **Update version numbers** according to semantic versioning

## Architecture Decisions

When proposing significant changes:

1. **Open an issue first** to discuss the approach
2. **Consider the ECS architecture** - New features should fit within the Entity Component System paradigm
3. **Think about extensibility** - How will this change affect future development?
4. **Performance matters** - Consider the impact on large library generation

## Types of Contributions

### Bug Reports

- Use the issue tracker
- Include minimal reproducible example
- Provide system information (OS, Rust version)
- Describe expected vs actual behavior

### Feature Requests

- Check existing issues first
- Explain the use case clearly
- Suggest implementation approach if possible
- Be open to alternative solutions

### Code Contributions

We welcome contributions in these areas:

- **New component types** (capacitors, inductors, etc.)
- **Additional manufacturers** support
- **New output formats** (Eagle, Cadence, etc.)
- **Performance improvements**
- **GUI enhancements**
- **Documentation improvements**

### Areas Requiring Discussion

These changes require prior discussion:

- Major architectural changes
- Changes to the ECS system design
- Breaking API changes
- New dependencies
- License-related changes

## Review Process

1. Maintainers will review PRs within 1-2 weeks
2. Address feedback constructively
3. Make requested changes in new commits (don't force-push during review)
4. Once approved, squash commits if requested

## Recognition

Contributors will be:
- Listed in the project's contributors file
- Credited in release notes for significant contributions
- Acknowledged in relevant documentation

## Questions?

If you have questions about contributing:

1. Check existing documentation
2. Look through closed issues/PRs
3. Open a discussion issue
4. Contact maintainers at atlantix-eda@proton.me

## License

By contributing to Atlantix-EDA, you agree that your contributions will be licensed under the GNU General Public License (GPL) as described in the LICENSE file.

---

Thank you for helping make Atlantix-EDA better for the electronics engineering community!