# proper input
POST {{host}}/duplicate
{
  "publisher_user_id": "{{publisher}}",
  "video_id": "7ec40a0b9aba4307a97e8666822ed563",
  "is_nsfw": false,
  "metadata": {
    "test": "value"
  }
}
HTTP 200

# when the video doesn't exist
POST {{host}}/duplicate
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
POST {{host}}/duplicate
`malformed json input`
HTTP 415

# missing required information
POST {{host}}/duplicate
{
  "video_id": "7ec40a0b9aba4307a97e8666822ed563",
  "is_nsfw": false,
  "metadata": {
    "test": "value"
  }
}
HTTP 422
