# proper input
POST {{host}}/storj/duplicate
Authorization: Bearer {{api_token}}
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "{{video_id}}",
  "is_nsfw": false,
  "metadata": {
    "test": "value"
  }
}
HTTP 200

# Segregate nsfw videos
POST {{host}}/storj/duplicate
Authorization: Bearer {{api_token}}
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "{{video_id}}",
  "is_nsfw": true,
  "metadata": {
    "test": "value"
  }
}
HTTP 200

# Without auth token
POST {{host}}/storj/duplicate
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "{{video_id}}",
  "is_nsfw": false,
  "metadata": {
    "test": "value"
  }
}
HTTP 401

# With wrong token
POST {{host}}/storj/duplicate
Authorization: Bearer wrongtoken
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "{{video_id}}",
  "is_nsfw": false,
  "metadata": {
    "test": "value"
  }
}
HTTP 401

# when the video doesn't exist
POST {{host}}/storj/duplicate
Authorization: Bearer {{api_token}}
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "an_id_that_doesnt_exist_on_cloudflare",
  "is_nsfw": false,
  "metadata": {
    "test": "value"
  }
}
HTTP 404

# malformed input
POST {{host}}/storj/duplicate
Authorization: Bearer {{api_token}}
`malformed json input`
HTTP 415

# missing required information
POST {{host}}/storj/duplicate
Authorization: Bearer {{api_token}}
{
  "video_id": "{{video_id}}",
  "is_nsfw": false,
  "metadata": {
    "test": "value"
  }
}
HTTP 422

# proper input
POST {{host}}/sia/duplicate
Authorization: Bearer {{api_token}}
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "{{video_id}}",
  "is_nsfw": false,
  "metadata": {
    "test": "value"
  }
}
HTTP 200

# Segregate nsfw videos
POST {{host}}/sia/duplicate
Authorization: Bearer {{api_token}}
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "{{video_id}}",
  "is_nsfw": true,
  "metadata": {
    "test": "value"
  }
}
HTTP 200

# Without auth token
POST {{host}}/sia/duplicate
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "{{video_id}}",
  "is_nsfw": false,
  "metadata": {
    "test": "value"
  }
}
HTTP 401

# With wrong token
POST {{host}}/sia/duplicate
Authorization: Bearer wrongtoken
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "{{video_id}}",
  "is_nsfw": false,
  "metadata": {
    "test": "value"
  }
}
HTTP 401

# when the video doesn't exist
POST {{host}}/sia/duplicate
Authorization: Bearer {{api_token}}
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "an_id_that_doesnt_exist_on_cloudflare",
  "is_nsfw": false,
  "metadata": {
    "test": "value"
  }
}
HTTP 404

# malformed input
POST {{host}}/sia/duplicate
Authorization: Bearer {{api_token}}
`malformed json input`
HTTP 415

# missing required information
POST {{host}}/sia/duplicate
Authorization: Bearer {{api_token}}
{
  "video_id": "{{video_id}}",
  "is_nsfw": false,
  "metadata": {
    "test": "value"
  }
}
HTTP 422

