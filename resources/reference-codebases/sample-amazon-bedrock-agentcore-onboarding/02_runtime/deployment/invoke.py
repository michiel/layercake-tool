import sys
import os
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from cost_estimator_agent.cost_estimator_agent import AWSCostEstimatorAgent
from bedrock_agentcore.runtime import BedrockAgentCoreApp

app = BedrockAgentCoreApp()

@app.entrypoint
def invoke(payload):
    user_input = payload.get("prompt")
    agent = AWSCostEstimatorAgent()

    # Batch
    return agent.estimate_costs(user_input)


if __name__ == "__main__":
    app.run()
