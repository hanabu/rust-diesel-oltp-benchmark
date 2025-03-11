import json
import subprocess

def lambda_handler(event, context):
    # Run rte
    concurrency = event.get('concurrency', 1)
    duration = event.get('duration', 60)
    endpoint = event['endpoint']
    subprocess.run(["./rte", "run", "-d", str(duration), "-c", str(concurrency), endpoint]),
    return {
        'statusCode': 200,
        'body': json.dumps('Finished')
    }
