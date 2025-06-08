---
sidebar_position: 3
---

## Proxy API endpoints

The following endpoints are all for use with Semcache operating in proxy mode, i.e. forwarding requests to your desired LLM provider.


### OpenAI format

#### `POST /v1/chat/completions`

### Request

- **Method**: `POST`
- **Headers**:
  | Header Name      | Value                | Required | Description                     |
  |------------------|----------------------|----------|---------------------------------|
  | `Content-Type`   | `application/json`   | yes       | Specifies that body is JSON     |
  | `x-llm-proxy-upstream`         | `https://full_path_to_desired_upstream.com/path`   | no       | Allows you to override the default upstream associated with this endpoint  |
  | `x-llm-proxy-host`         | `https://host_to_override_default.com`   | no       | Allows for just overriding the host part of the url    |
  | `x-llm-proxy-prompt`         | `$.json_path_of_prompt_field`   | no       | Allows for overriding the default prompt location   |

- **Body** (`application/json`):
  ```json
    { "model": "gpt-4o", "messages": [{"role": "user", "content": "prompt?"}]}


### Anthropic format

#### `POST /v1/messages`

### Request

- **Method**: `POST`
- **Headers**:
  | Header Name      | Value                | Required | Description                     |
  |------------------|----------------------|----------|---------------------------------|
  | `Content-Type`   | `application/json`   | yes       | Specifies that body is JSON     |
  | `x-llm-proxy-upstream`         | `https://full_path_to_desired_upsteam.com/path`   | no       | Allows you to override the default upstream associated with this endpoint  |
  | `x-llm-proxy-host`         | `https://host_to_override_default.com`   | no       | Allows for just overriding the host part of the url    |
  | `x-llm-proxy-prompt`         | `$.json_path_of_prompt_field`   | no       | Allows for overriding the default prompt location   |

- **Body** (`application/json`):
  ```json
  {
    "model": "claude-opus-4-20250514",
    "max_tokens": 1024,
    "messages": [
        {"role": "user", "content": "Hello, world"}
    ]
  }

### Generic format

#### `POST /semcache/v1/chat/completions`

### Request

- **Method**: `POST`
- **Headers**:
  | Header Name      | Value                | Required | Description                     |
  |------------------|----------------------|----------|---------------------------------|
  | `Content-Type`   | `application/json`   | yes       | Specifies that body is JSON     |
  | `x-llm-proxy-upstream`         | `https://full_path_to_desired_upsteam.com/path`   | yes       | Set the upstream you want us to forward requests to |
  | `x-llm-proxy-host`         | `https://host_to_override_default.com`   | no       | Allows for just overriding the host part of the url    |
  | `x-llm-proxy-prompt`         | `$.json_path_of_prompt_field`   | yes       | Set the jsonpath of cache key   |

- **Body** (`application/json`):
  ```json
    { "query": "string to use as key for cache lookup"}


### Headers

If you are curious about when you might want to set the `x-llm-` headers, refer to [LLM Providers & Tools](https://docs.semcache.io/docs/llm-providers-tools).

Other than this, the headers sent to the proxy will be forwarded to the upstream on outgoing calls. This means that you may need to set authentication headers, or other metadata related headers needed for your upstream to properly understand your request.

### Outgoing request body
In the event of a cache miss, the incoming request body will be sent as is to the proxy upstream.

### Response
In the event of a cache miss, we will return the response unmodified from the proxy upstream.

In the event of a cache hit, we will return the stored value matched to the key specified by the `x-llm-proxy-prompt` (or the default associated with the specific route). If this is from another LLM provider, you need to be able to handle their format on your end.



## Cache aside API endpoints

We also expose endpoints that allow you to utilize semcache in a cache-aside manner.


### Write to cache

#### `PUT /v1/semcache/put`

### Request

- **Method**: `PUT`
- **Headers**:
  | Header Name      | Value                | Required | Description                     |
  |------------------|----------------------|----------|---------------------------------|
  | `Content-Type`   | `application/json`   | yes       | Specifies that body is JSON     |

- **Body** (`application/json`):
  ```json
    { "key": "What is the capital of France?", "data": "Paris"}

### Response

- **Status Codes**:

  | Code | Meaning                   | When It Occurs                                   |
  |------|---------------------------|--------------------------------------------------|
  | 200  | OK                        | Cache entry was successfully written or updated |
  | 500  | Internal Server Error     | An unexpected server error occurred             |

### Read from cache

#### `PUT /v1/semcache/put`

### Request

- **Method**: `PUT`
- **Headers**:
  | Header Name      | Value                | Required | Description                     |
  |------------------|----------------------|----------|---------------------------------|
  | `Content-Type`   | `application/json`   | yes       | Specifies that body is JSON     |

- **Body** (`application/json`):
  ```json
    { "key": "What is the capital of France?"}

- **Status Codes**:

  | Code | Meaning                   | When It Occurs                                   |
  |------|---------------------------|--------------------------------------------------|
  | 200  | OK                        | Cache entry was successfully written or updated |
  | 404  | Not Found               | No corresponding cache entry was found |
  | 500  | Internal Server Error     | An unexpected server error occurred             |

- **Body** (`application/json`):
  ```string
   "Paris" 
