# FeedTape Backend - Clarification Questions

After reviewing the project context and OpenAPI specification, I have some questions to better understand your requirements and help build the backend effectively:

## Architecture & Infrastructure

1. **Deployment Strategy**: You mentioned Railway.app for hosting. Are you planning to use Railway's PostgreSQL offering, or do you have another database solution in mind? Do you need help setting up the Railway deployment pipeline?
Answer: Yes I will use it but by now lets focus on local development. I would like to use docker compose to run postgres locally You need to create the docker-compose file.

2. **Caching Layer**: The context mentions adding Redis in Phase 2. Should the initial implementation include hooks/interfaces for future caching, or keep it minimal for MVP?
Answer: No redis, keep it simple for now

3. **Environment Configuration**: How do you plan to manage secrets (AWS credentials, OAuth keys, JWT secrets)? Through Railway's environment variables or another solution?
Answer: Lets dont worry about railway for now. Let's use envionrment variables. I usually use direnv for local development.

## Authentication & Security

4. **OAuth Providers**: Which OAuth provider would you like to implement first? Should I focus on one initially (e.g., Google) or implement all three (Apple, Google, GitHub) from the start?
Answer: I would say apple first, then google and github last. But I would like to have all 3 before end of MVP.

5. **JWT Strategy**: The spec mentions 1-hour access tokens and 30-day refresh tokens. Do you want automatic token refresh handled on the backend, or should the mobile app manage this?
Answer: the mobile app.

6. **Rate Limiting**: What specific rate limits do you want to implement per endpoint? Should rate limiting be user-based or IP-based?
Answer: No rate limit for now, keep it simple. We will need to track the usage per user and enforce the limits. For example we will return an error if you have surpassed your quota when requesting TTS.
We need to track the usage in characters and minutes. We will define the limits on later stages.

## TTS Integration

7. **AWS Polly Configuration**: Do you already have an AWS account set up? Which specific Polly voices do you want to support initially (Lucia, Sergio, Mia, Andrés as mentioned)?
Answer: Yes I have an aws account. I would like to support all the voices you mentioned. But eventually we will save one male and female per language and will support as many languages as possible (EN, ES, FR, DE, IT, PT)

8. **Audio Format**: The API returns audio/mpeg and audio/ogg. Which format should be the default? Should the backend support format conversion or rely on Polly's native outputs?
Answer: rely on polly outputs. We will use mp3 by default.

9. **Usage Tracking**: How granular should usage tracking be? Per-request logging, or aggregated daily summaries?
Answer: I believe that we can start tracking characters of the article we want to convert to audio. It would be good to track minutes of audio if possible but keep it simple. Lets assume a ratio between characters and minutes if needed. We need to track the amount of characters per user per day and per month in the database.

## Database Schema

10. **User Data**: Beyond email and OAuth provider info, what user data should we store? Display name? Profile picture? Timezone for usage reset calculations?
    Answer: no more data, no user profile. Just id, email, auth provider, created at and settings (a json object)

11. **Feed Storage**: Should we store any metadata about feeds (like last fetch time, article count) even though fetching happens client-side?
Answer, not by now.

12. **Subscription Management**: How will Pro subscriptions be handled? Through Apple/Google in-app purchases only, or also web payments (Stripe)?
    Answer: through apple in in app purchases. Only ios by now.

## Business Logic

13. **Free Tier Limits**: The context mentions 20 minutes/day and 30,000 characters. Which limit takes precedence? Should we track both?
Answer: Polly works with characters, lets use characters then and assume a ratio of characters per minute. This is how I want the subscriptions to work. Lets assume that a minute of audio are 1000 characters. 
- Free: 20 minutes per day during 7 days. Then you need to convert to pro to keep using it. 20 minutes are 20000 characters.
- Pro: 200 minutes per day

14. **Language Detection**: For the "auto" language option, should we use a library for language detection or rely on Polly's capabilities?
Lets use a library, I read about lingua-rs but you can choose any other.

15. **Error Handling**: How verbose should error responses be? Should we include stack traces in development mode?
Yes, stack traces in development mode. About logs I will take a look if Railway offers anything for logs. But by default lets be as verbose as possible in dev (log requests, responses, errors, ...)

## Development Priorities

16. **MVP Scope**: Which endpoints are absolutely essential for your first release? Can we defer some features (like usage history, subscription validation) to post-MVP?
 Answer: I believe there is no much to differ. All endpoints are important. But if needed we can defer usage history.

17. **Testing Strategy**: What level of test coverage are you aiming for? Unit tests only, or also integration tests with real AWS calls?
Would we good to have integration tests that mock AWS calls. We can use something like moto for that.

18. **Monitoring**: Beyond basic health checks, what metrics are important to track? Request latency? TTS generation time? Error rates?
Answer: just healtcheck by now

## Technical Preferences

19. **Rust Web Framework**: The context mentions Axum. Are you committed to Axum, or open to alternatives like Actix-web or Rocket?
Answer: I've never used any of them. Let's choose the more popular one.

20. **Database Migrations**: How should we handle schema migrations? Using a tool like sqlx migrations or diesel?
Answer: by now no migrations. Lets use a single sql file to create the tables.

21. **API Versioning**: The OpenAPI spec shows v3.0.0. How do you want to handle future API versions? URL versioning (/v3/api) or header-based?
Answer: No versioning by now. We will see when we need it.

## Next Steps

22. **Development Environment**: Should I set up Docker containers for local development, or rely on native Rust/PostgreSQL installation?
Answer: Yes, use docker compose. for postgres. For rust you can use native installation.

23. **Code Organization**: Do you have preferences for project structure? Domain-driven design, layered architecture, or something else?
Answer I like DDD but for an MPV I believe we can keep it simple and go with a layered architecture.

24. **Implementation Order**: Would you prefer to start with authentication, the TTS proxy, or the basic CRUD operations for feeds?
Answerf I dont care

Please answer the questions that are most important to you, and we can use defaults or best practices for the others. This will help me build exactly what you need for FeedTape's backend.
