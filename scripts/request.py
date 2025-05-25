#!/usr/bin/env python3

import requests
import json
import sys
import time


def make_request(provider, api_key, prompt, full_response=False, proxy=True):
    start_time = time.time()

    if proxy:
        base_url = "http://localhost:8080"
    else:
        if provider == "openai":
            base_url = "https://api.openai.com/v1"
        elif provider == "deepseek":
            base_url = "https://api.deepseek.com/v1"
        elif provider == "anthropic":
            base_url = "https://api.anthropic.com/v1"

    headers = {"Content-Type": "application/json", "Authorization": f"Bearer {api_key}"}

    if proxy:
        if provider == "openai":
            headers["host"] = "api.openai.com"
            headers["X-LLM-Proxy-Upstream"] = (
                "https://api.openai.com/v1/chat/completions"
            )
            url = f"{base_url}/chat/completions"
        elif provider == "deepseek":
            headers["host"] = "api.deepseek.com"
            headers["X-LLM-Proxy-Upstream"] = (
                "https://api.deepseek.com/v1/chat/completions"
            )
            url = f"{base_url}/chat/completions"
        elif provider == "anthropic":
            headers["host"] = "api.anthropic.com"
            headers["X-LLM-Proxy-Upstream"] = "https://api.anthropic.com/v1/messages"
            url = f"{base_url}/messages"
    else:
        if provider == "openai":
            url = f"{base_url}/chat/completions"
        elif provider == "deepseek":
            url = f"{base_url}/chat/completions"
        elif provider == "anthropic":
            url = f"{base_url}/messages"

    if provider == "anthropic":
        headers["anthropic-version"] = "2023-06-01"
        data = {
            "model": "claude-3-sonnet-20240229",
            "max_tokens": 1000,
            "messages": [{"role": "user", "content": prompt}],
        }
    else:
        if provider == "openai":
            model = "gpt-4o"
        elif provider == "deepseek":
            model = "deepseek-chat"

        data = {"model": model, "messages": [{"role": "user", "content": prompt}]}

    response = requests.post(url, headers=headers, json=data)
    end_time = time.time()
    duration_ms = (end_time - start_time) * 1000

    print(f"⏱️  Request completed in {duration_ms:.0f}ms")
    print("=" * 50)

    if full_response:
        return response.json()
    else:
        result = response.json()
        if provider == "anthropic":
            return result["content"][0]["text"]
        else:
            return result["choices"][0]["message"]["content"]


if __name__ == "__main__":
    if len(sys.argv) < 4:
        print(
            "Usage: python script.py <provider> <api_key> <prompt> [full_response] [proxy]"
        )
        sys.exit(1)

    provider = sys.argv[1]
    api_key = sys.argv[2]
    prompt = sys.argv[3]
    full_response = sys.argv[4].lower() == "true" if len(sys.argv) > 4 else False
    proxy = sys.argv[5].lower() == "true" if len(sys.argv) > 5 else True

    result = make_request(provider, api_key, prompt, full_response, proxy)
    print(result)
