# Contributing to DAO Reputation Scoreboard

We welcome contributions from the community! This document provides guidelines for contributing to the DAO Reputation Scoreboard project.

## 🤝 How to Contribute

### Types of Contributions

- **Bug Reports**: Report issues and bugs
- **Feature Requests**: Suggest new features or improvements
- **Code Contributions**: Submit bug fixes or new features
- **Documentation**: Improve documentation and examples
- **Testing**: Add or improve test coverage
- **Security**: Report security vulnerabilities

## 🚀 Getting Started

### Development Setup

1. **Fork the Repository**
   ```bash
   git clone https://github.com/your-username/dao-reputation-scoreboard.git
   cd dao-reputation-scoreboard
   ```

2. **Install Dependencies**
   ```bash
   npm install
   ```

3. **Set Up Development Environment**
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install Solana CLI
   sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
   
   # Install Anchor
   npm install -g @coral-xyz/anchor-cli@0.29.0
   ```

4. **Build and Test**
   ```bash
   anchor build
   anchor test
   ```

### Development Workflow

1. **Create a Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make Your Changes**
   - Write clean, well-commented code
   - Follow existing code style and patterns
   - Add tests for new functionality

3. **Test Your Changes**
   ```bash
   anchor test
   npm run lint
   ```

4. **Commit Your Changes**
   ```bash
   git add .
   git commit -m "feat: add new feature description"
   ```

5. **Push and Create Pull Request**
   ```bash
   git push origin feature/your-feature-name
   ```

## 📝 Coding Standards

### Rust Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Use meaningful variable and function names
- Add comprehensive comments for complex logic
- Include proper error handling
- Write unit tests for new functions

```rust
/// Calculate user's total reputation score with category weights
/// 
/// # Arguments
/// * `category_points` - Points in each reputation category
/// * `weights` - Weight multipliers for each category (basis points)
/// 
/// # Returns
/// * `u64` - Calculated total score
pub fn calculate_total_score(
    category_points: &[u64; 4], 
    weights: &[u16; 4]
) -> Result<u64> {
    let mut total = 0u64;
    
    for (i, &points) in category_points.iter().enumerate() {
        let weighted_points = points
            .checked_mul(weights[i] as u64)
            .ok_or(ReputationError::NumericalOverflow)?;
        
        total = total
            .checked_add(weighted_points)
            .ok_or(ReputationError::NumericalOverflow)?;
    }
    
    Ok(total / 10000) // Normalize basis points
}
```

### TypeScript Code Style

- Use TypeScript strict mode
- Follow consistent naming conventions
- Add type annotations for public APIs
- Include JSDoc comments for functions

```typescript
/**
 * Initialize a new user reputation account
 * @param program - Anchor program instance
 * @param userKeypair - User's keypair
 * @returns Transaction signature
 */
export async function initializeUserReputation(
  program: Program<DaoReputationScoreboard>,
  userKeypair: Keypair
): Promise<string> {
  const [userReputationPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("user_reputation"), userKeypair.publicKey.toBuffer()],
    program.programId
  );

  return await program.methods
    .initializeUserReputation()
    .accounts({
      userReputation: userReputationPDA,
      user: userKeypair.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([userKeypair])
    .rpc();
}
```

## 🧪 Testing Guidelines

### Test Categories

1. **Unit Tests** - Test individual functions
2. **Integration Tests** - Test instruction interactions
3. **Edge Case Tests** - Test boundary conditions
4. **Security Tests** - Test access controls and validation
5. **Performance Tests** - Test gas usage and efficiency

### Writing Tests

```typescript
describe("Reputation Calculation", () => {
  it("should calculate total score correctly with weights", async () => {
    // Setup
    const categoryPoints = [1000, 800, 600, 400];
    const weights = [3000, 2500, 2500, 2000]; // Basis points
    
    // Execute
    const totalScore = calculateTotalScore(categoryPoints, weights);
    
    // Verify
    expect(totalScore).to.equal(2080); // Expected weighted sum
  });

  it("should handle overflow gracefully", async () => {
    // Test with maximum values to trigger overflow
    const maxPoints = [Number.MAX_SAFE_INTEGER, 0, 0, 0];
    const weights = [10000, 0, 0, 0];
    
    try {
      calculateTotalScore(maxPoints, weights);
      expect.fail("Should have thrown overflow error");
    } catch (error) {
      expect(error.message).to.include("NumericalOverflow");
    }
  });
});
```

### Test Requirements

- **Coverage**: Aim for >90% code coverage
- **Edge Cases**: Test boundary conditions and error paths
- **Security**: Test all access controls and validations
- **Performance**: Verify gas usage is reasonable
- **Documentation**: Include clear test descriptions

## 🔒 Security Guidelines

### Security Best Practices

1. **Input Validation**
   - Validate all user inputs
   - Check numerical bounds
   - Sanitize string inputs

2. **Access Controls**
   - Verify admin permissions
   - Check account ownership
   - Validate signer requirements

3. **Overflow Protection**
   - Use safe math operations
   - Check for numerical overflows
   - Handle edge cases gracefully

4. **PDA Security**
   - Use proper seed derivation
   - Verify account relationships
   - Check account initialization

### Security Review Process

1. **Self Review**
   - Review your own code for security issues
   - Test edge cases and error conditions
   - Verify access controls

2. **Peer Review**
   - All PRs require review from maintainers
   - Focus on security implications
   - Test security-critical changes

3. **Security Audit**
   - Major changes undergo security review
   - External audits for significant updates
   - Bug bounty program for critical issues

## 📋 Pull Request Process

### PR Requirements

- [ ] **Clear Description** - Explain what changes were made and why
- [ ] **Tests Added** - Include tests for new functionality
- [ ] **Documentation Updated** - Update relevant documentation
- [ ] **Code Style** - Follow project coding standards
- [ ] **Security Review** - Consider security implications
- [ ] **Breaking Changes** - Document any breaking changes

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing completed

## Security Considerations
- [ ] Access controls verified
- [ ] Input validation added
- [ ] No new security vulnerabilities

## Documentation
- [ ] Code comments added
- [ ] README updated if needed
- [ ] API documentation updated
```

### Review Process

1. **Automated Checks**
   - Code builds successfully
   - All tests pass
   - Linting checks pass

2. **Manual Review**
   - Code quality and style
   - Security implications
   - Test coverage adequacy

3. **Approval and Merge**
   - At least one maintainer approval
   - All checks passing
   - Squash and merge preferred

## 🐛 Bug Reports

### Bug Report Template

```markdown
## Bug Description
Clear description of the bug

## Steps to Reproduce
1. Step one
2. Step two
3. Step three

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- Solana version:
- Anchor version:
- Node.js version:
- Operating System:

## Additional Context
Any other relevant information
```

### Priority Levels

- **Critical** - Security vulnerabilities, data loss
- **High** - Major functionality broken
- **Medium** - Minor functionality issues
- **Low** - Cosmetic issues, improvements

## 💡 Feature Requests

### Feature Request Template

```markdown
## Feature Description
Clear description of the proposed feature

## Use Case
Why is this feature needed? What problem does it solve?

## Proposed Solution
How should this feature work?

## Alternatives Considered
What other approaches were considered?

## Additional Context
Any other relevant information
```

## 🏆 Recognition

### Contributors

We recognize contributors in several ways:

- **README Credits** - Listed in project README
- **Changelog Attribution** - Credited in release notes
- **Discord Recognition** - Special roles in community Discord
- **Conference Opportunities** - Speaking opportunities at events

### Contribution Types

- **Code Contributors** - Bug fixes and new features
- **Documentation Contributors** - Guides and API docs
- **Community Contributors** - Support and engagement
- **Security Contributors** - Vulnerability reports and fixes

## 📞 Getting Help

### Communication Channels

- **GitHub Issues** - Bug reports and feature requests
- **GitHub Discussions** - General questions and discussions
- **Discord** - Real-time community chat
- **Email** - Security vulnerabilities and private matters

### Resources

- **Documentation** - README.md and deployment guides
- **Examples** - Reference implementations
- **Tests** - Usage patterns and edge cases
- **Code Comments** - Inline documentation

## 📜 Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of:

- Background and identity
- Experience level
- Nationality and ethnicity
- Religion and political views
- Sexual orientation and gender identity

### Expected Behavior

- **Be Respectful** - Treat others with courtesy and respect
- **Be Collaborative** - Work together constructively
- **Be Professional** - Maintain professional communication
- **Be Inclusive** - Welcome newcomers and diverse perspectives

### Unacceptable Behavior

- Harassment or discrimination
- Trolling or inflammatory comments
- Personal attacks or insults
- Spam or irrelevant content

### Enforcement

Violations of the code of conduct should be reported to project maintainers. We reserve the right to remove contributors who violate these guidelines.

## 🎯 Development Roadmap

### Current Priorities

1. **Core Stability** - Bug fixes and security improvements
2. **Performance** - Gas optimization and efficiency
3. **Documentation** - Comprehensive guides and examples
4. **Testing** - Improved test coverage and edge cases

### Future Features

1. **Cross-chain Integration** - Bridge reputation across chains
2. **Advanced Analytics** - Detailed reputation insights
3. **Governance Integration** - Direct proposal system integration
4. **Mobile Support** - Mobile-friendly interfaces

### How to Get Involved

- **Check Issues** - Look for "good first issue" labels
- **Join Discussions** - Participate in feature planning
- **Propose Features** - Submit detailed feature requests
- **Review PRs** - Help review community contributions

---

Thank you for contributing to the DAO Reputation Scoreboard! Your contributions help make decentralized governance more effective and transparent. 🚀