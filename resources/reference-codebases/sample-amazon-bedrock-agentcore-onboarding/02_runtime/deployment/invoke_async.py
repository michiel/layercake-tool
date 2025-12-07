import sys
import os
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from cost_estimator_agent.cost_estimator_agent import AWSCostEstimatorAgent
from bedrock_agentcore.runtime import BedrockAgentCoreApp

app = BedrockAgentCoreApp()

@app.entrypoint
async def invoke(payload):
    user_input = payload.get("prompt")
    agent = AWSCostEstimatorAgent()
    stream = agent.estimate_costs_stream(user_input)
    async for event in stream:
        yield (event)


if __name__ == "__main__":
    app.run()
