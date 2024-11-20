import json
import subprocess

def lambda_handler(event, context):
    # Run rte
    subprocess.run(["./rte", "run", "-d", "30", "-c", "1", "https://npolvz556254fqb3l4vkgu5mqe0iohsf.lambda-url.ap-northeast-1.on.aws/"],
    return {
        'statusCode': 200,
        'body': json.dumps('Finished')
    }

