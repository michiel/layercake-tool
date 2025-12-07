# Amazon Bedrock AgentCore Development Guide

## Project Overview
This project provides practical, simple, and runnable code examples for onboarding developers to Amazon Bedrock AgentCore. The goal is to publish `agentcore_blog_ja.md` with perfect examples.

## Key Development Principles

### 1. Runnable Code First
- Always refer to the latest AWS documentation before implementation
- Verify all code examples work with current AWS SDK versions
- Test implementations against live AWS services
- Include complete, executable examples (not fragments)
- Use meaningful, descriptive filenames

### 2. Practical Implementation
- Add comprehensive comments and logging for monitoring/debugging
- Include error handling patterns and troubleshooting guidance
- Provide real-world use cases and scenarios
- Create progressive learning paths with numbered sequences (01-, 02-, 03-)

### 3. Simple and Sophisticated
- Keep code simple to minimize learning cost
- Use clear, descriptive variable and function names
- Follow consistent coding patterns
- Prefer flat, simple directory structures

## Code Quality Requirements

### Must-Have Features
- **No dummy data ever** - Use real API connections and service integrations
- **Proper resource cleanup** - Use try/finally blocks for resource management
- **Meaningful naming** - Use descriptive filenames like `cost_estimator_agent.py`
- **Separated configuration** - Extract prompts and constants to pass linting tools
- **Follow language conventions** - Use underscores for Python package compatibility

### Error Handling
- Implement proper exception handling patterns
- Include retry logic with exponential backoff
- Provide clear error messages and resolution steps
- Handle common AWS service errors

### Logging and Monitoring
- Implement structured logging for all operations
- Include debug-level logging for development
- Provide CloudWatch integration examples
- Show how to monitor AgentCore performance

## Testing Commands
When implementing code, run these commands to ensure code quality:
- **Linting**: `ruff check` (if Python code)
- **Type checking**: Check for appropriate type checking commands in the project
- **Testing**: Look for test scripts in the project structure

## Execution Environment
- Use `uv run` commands for Python execution
- Match actual project setup and directory structures
- List actual AWS permissions and service requirements
- Reference specific SDK versions

## Documentation Standards
Every code example must include:
- Purpose and use case explanation
- Required AWS permissions and setup
- Step-by-step execution instructions
- Expected outputs and results
- Common troubleshooting scenarios

## Resource Management
- Always implement cleanup for AWS resources
- Follow service-specific cleanup patterns (e.g., AgentCore Code Interpreter requires explicit session stopping)
- Document resource lifecycle and consequences of not cleaning up
- Handle multi-service coordination complexity

## Implementation Checklist
Before completing any code example:
- [ ] Code runs successfully with latest AWS SDK
- [ ] All dependencies are clearly documented
- [ ] Comprehensive comments explain each major step
- [ ] Logging is implemented for debugging
- [ ] Error handling covers common scenarios
- [ ] No dummy or placeholder data
- [ ] Proper resource cleanup with try/finally blocks
- [ ] Meaningful filenames that indicate purpose
- [ ] Separated configuration for linting compliance
- [ ] Correct execution commands using `uv run`
- [ ] Service-specific patterns followed

## Project Structure
- Start simple, evolve as needed
- Use numbered sequences for learning paths
- One concept per directory
- Documentation-first approach (create README.md files first)
- Meaningful names over conventions

Remember: The goal is to make Amazon Bedrock AgentCore accessible to developers of all skill levels through practical, runnable, and well-documented examples.