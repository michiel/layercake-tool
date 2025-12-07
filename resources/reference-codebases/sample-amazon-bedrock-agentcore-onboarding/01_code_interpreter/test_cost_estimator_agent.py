#!/usr/bin/env python3
"""Simple test for AWS Cost Estimation Agent"""

import asyncio
import argparse
from cost_estimator_agent.cost_estimator_agent import AWSCostEstimatorAgent

async def test_streaming(architecture: str, verbose: bool = True):
    """Test streaming cost estimation following Strands best practices"""
    if verbose:
        print("\n🔄 Testing streaming cost estimation...")
    agent = AWSCostEstimatorAgent()
    
    # Use provided test case or default
    
    try:
        total_chunks = 0
        total_length = 0
        
        async for event in agent.estimate_costs_stream(architecture):
            if "data" in event:
                # According to Strands documentation, each event["data"] should contain
                # only the new delta content, so we can print it directly
                chunk_data = str(event["data"])
                if verbose:
                    print(chunk_data, end="", flush=True)
                
                # Track metrics for debugging
                total_chunks += 1
                total_length += len(chunk_data)
                
            elif "error" in event:
                if verbose:
                    print(f"\n❌ Streaming error: {event['data']}")
                return False
        
        if verbose:
            print(f"\n📊 Streaming completed: {total_chunks} chunks, {total_length} total characters")
        return total_length > 0
        
    except Exception as e:
        if verbose:
            print(f"❌ Streaming test failed: {e}")
        return False

def test_regular(architecture: str = "One EC2 t3.micro instance running 24/7", verbose: bool = True):
    """Test regular (non-streaming) cost estimation"""
    if verbose:
        print("📄 Testing regular cost estimation...")
    agent = AWSCostEstimatorAgent()
    
    # Use provided test case or default
    
    try:
        result = agent.estimate_costs(architecture)
        if verbose:
            print(f"📊 Regular response length: {len(result)} characters")
            print(f"Result preview: {result[:150]}...")
        return len(result) > 0
    except Exception as e:
        if verbose:
            print(f"❌ Regular test failed: {e}")
        return False


def parse_arguments():
    """Parse command line arguments"""
    parser = argparse.ArgumentParser(description='Test AWS Cost Estimation Agent')
    
    parser.add_argument(
        '--architecture', 
        type=str, 
        default="One EC2 t3.micro instance running 24/7",
        help='Architecture description to test (default: "One EC2 t3.micro instance running 24/7")'
    )
    
    parser.add_argument(
        '--tests',
        nargs='+',
        choices=['regular', 'streaming', 'debug'],
        default=['regular'],
        help='Which tests to run (default: regular)'
    )
    
    parser.add_argument(
        '--verbose',
        action='store_true',
        default=True,
        help='Enable verbose output (default: True)'
    )
    
    parser.add_argument(
        '--quiet',
        action='store_true',
        help='Disable verbose output'
    )
    
    return parser.parse_args()

async def main():
    args = parse_arguments()
    
    # Handle verbose flag
    verbose = args.verbose and not args.quiet
    
    print("🚀 Testing AWS Cost Agent")
    if verbose:
        print(f"Architecture: {args.architecture}")
        print(f"Tests to run: {', '.join(args.tests)}")
    
    results = {}
    
    # Run selected tests
    if 'regular' in args.tests:
        results['regular'] = test_regular(args.architecture, verbose)
    
    if 'streaming' in args.tests:
        results['streaming'] = await test_streaming(args.architecture, verbose)
    
    # Print results
    if verbose:
        print("\n📋 Test Results:")
        for test_name, success in results.items():
            status = '✅ PASS' if success else '❌ FAIL'
            print(f"   {test_name.capitalize()} implementation: {status}")
        
        if all(results.values()):
            print("🎉 All tests completed successfully!")
        else:
            print("⚠️ Some tests failed - check logs above")
    
    # Return exit code based on results
    return 0 if all(results.values()) else 1

if __name__ == "__main__":
    import sys
    exit_code = asyncio.run(main())
    sys.exit(exit_code)
