# FeedTape - Project Context Document

## 🎯 Product Vision

FeedTape is a mobile app that converts RSS feed articles into high-quality audio using text-to-speech, enabling users to listen to their favorite blogs while commuting, exercising, or doing other activities. Think of it as "podcasts for written content."

### Core Value Proposition
- **Transform any RSS feed into a personal podcast**
- **Listen to articles instead of reading them**
- **Premium voice quality using neural TTS**
- **Dead-simple user experience**

### Target Users
- Commuters who want to consume content while driving
- People who prefer audio learning
- Multitaskers who want to stay informed while doing other activities
- Non-native speakers who find listening easier than reading

## 📱 Product Features

### MVP Features (Current Focus)
1. **RSS Feed Management** - Add/remove RSS feed URLs
2. **Audio Playback** - Convert articles to speech and play them
3. **Voice Selection** - Choose from multiple Spanish voices (Lucia, Sergio, Mia, Andrés)
4. **Playback Controls** - Play, pause, skip, speed adjustment
5. **Free Tier** - 20 minutes of audio per day
6. **Pro Tier** - Unlimited audio, higher quality voices

### Future Features (Post-MVP)
- Background playback and downloads
- Cross-device sync
- Smart playlists and auto-play
- AI-powered summaries
- Multiple language support
- Social features (share audio clips)

## 🛠 Technical Stack

### Mobile App
- **Framework**: React Native + Expo
- **Language**: TypeScript
- **State Management**: Local state + AsyncStorage
- **Audio Player**: expo-av
- **RSS Parsing**: Local XML parsing
- **Platform**: iOS first, Android later

### Backend API
- **Language**: Rust
- **Framework**: Axum web framework
- **Database**: PostgreSQL
- **Hosting**: Railway.app ($5/month)
- **Authentication**: OAuth2 (Apple/Google/GitHub) + JWT
- **TTS Service**: Amazon Polly (via AWS SDK)

### Infrastructure
- **Deployment**: Railway (auto-deploy from GitHub)
- **SSL**: Provided by Railway
- **Monitoring**: Basic health checks
- **Logging**: Structured logging to stdout

## 🏗 Architecture Decisions

### 1. Frontend RSS Fetching vs Backend Fetching
**Decision**: Frontend fetching
**Why**:
- Native mobile apps don't have CORS restrictions
- Simpler backend (10x less code)
- Privacy-first (reading habits stay on device)
- Lower server costs
- Can always add backend fetching later

**Trade-offs**:
- Slower initial app load (2-3 seconds to fetch feeds)
- No push notifications for new articles
- No cross-device article sync
- Each user converts same articles separately

### 2. Authentication Strategy
**Decision**: OAuth2-only (no passwords)
**Why**:
- No password management complexity
- Users trust Apple/Google more
- Faster onboarding
- Industry standard

**Implementation**:
- OAuth providers: Apple, Google, GitHub
- JWT tokens (1 hour) + refresh tokens (30 days)
- Stateless authentication

### 3. TTS Architecture
**Decision**: Backend proxy to Amazon Polly
**Why**:
- Secure (AWS credentials never exposed)
- Usage tracking and limits enforcement
- Potential for caching popular content
- Single point for optimization

**Alternative considered**: Direct Polly access from app
**Rejected because**: Security risk, no usage control

### 4. API Design
**Decision**: Minimal REST API with simplified endpoints
**Why**:
- Faster to implement (days vs weeks)
- Easier to maintain
- Clear separation of concerns

**Endpoints**:
- `/api/me` - User info, settings, subscription (single endpoint)
- `/api/feeds` - CRUD for feed URLs
- `/api/tts/synthesize` - Text to speech conversion
- Auth endpoints for OAuth flow

### 5. Monetization Strategy
**Decision**: Freemium with hard limits
**Tiers**:
- **Free**: 20 minutes/day, 3 feeds
- **Pro**: Unlimited audio, unlimited feeds

**Why**:
- Sustainable unit economics
- Clear value proposition for upgrade
- Generous enough free tier to hook users

### 6. Technology Choices

#### Why Rust for Backend?
- 10x lower memory usage than Node.js
- Can handle 1000+ users on $5/month server
- Type safety prevents runtime errors
- Excellent async performance
- Great PostgreSQL integration

#### Why Railway for Hosting?
- Zero DevOps required
- PostgreSQL included
- Automatic SSL certificates
- GitHub auto-deploy
- Cheaper than Heroku, simpler than AWS

#### Why Amazon Polly for TTS?
- Best Spanish neural voices (Lucia, Sergio)
- Pay-per-use pricing
- Fast generation (<500ms)
- Reliable API
- Wide language support for future

#### Why React Native + Expo?
- Single codebase for iOS/Android
- Fast development cycle
- Great audio libraries
- Native performance where needed
- Easy app store deployment

## 📊 Technical Specifications

### Performance Requirements
- TTS latency: < 1 second to first audio byte
- API response time: < 200ms
- Feed parsing: < 100ms per feed
- App launch to playable: < 3 seconds

### Scalability Plan
- **Phase 1** (0-100 users): Single Railway instance
- **Phase 2** (100-1000 users): Add Redis caching
- **Phase 3** (1000+ users): Multiple instances + CDN

### Security Measures
- JWT tokens with short expiry
- Rate limiting per user
- Input sanitization
- HTTPS only
- AWS credentials only on backend

## 💰 Cost Analysis

### Per User Economics (Monthly)
**Free User**:
- Polly API: ~€0.50 (limited usage)
- Hosting allocation: ~€0.05
- Net: -€0.55

**Pro User**:
- Revenue: €4.99
- Polly API: ~€2.00 (heavy usage)
- Hosting allocation: ~€0.10
- Net: +€2.89

**Break-even**: ~17% conversion rate

### Infrastructure Costs (Monthly)
- **100 users**: €5 (Railway) + €10 (Polly) = €15
- **1000 users**: €10 (Railway) + €50 (Polly) = €60
- **10000 users**: €50 (Railway + Redis) + €400 (Polly) = €450

## 🚀 Development Roadmap

### Phase 1: MVP (Current)
- [x] Tech spike with iOS native TTS
- [x] Tech spike with Amazon Polly
- [x] API specification
- [ ] Rust backend implementation
- [ ] OAuth integration
- [ ] Mobile app polish
- [ ] App Store submission

### Phase 2: Growth
- [ ] Android support
- [ ] Background downloads
- [ ] Push notifications
- [ ] Analytics dashboard

### Phase 3: Scale
- [ ] AI summaries
- [ ] Social features
- [ ] Web player
- [ ] B2B offerings

## 📝 Key Design Principles

1. **Simplicity First** - Every feature must be obvious to use
2. **Privacy-Focused** - User data stays on device when possible
3. **Performance** - Audio should start playing within 1 second
4. **Reliability** - Offline mode is a first-class citizen
5. **Sustainability** - Unit economics must work at scale

## 🎓 Lessons Learned

### From Tech Spikes
- iOS native TTS works but has quality limitations
- Amazon Polly provides superior voice quality
- CORS is not an issue for native mobile apps
- Users need audio to work in silent mode
- Simple UI beats feature-rich UI

### Architecture Insights
- Frontend fetching dramatically simplifies the backend
- OAuth-only authentication removes entire categories of problems
- Hard usage limits are essential for freemium models
- Rust backend can handle surprising load on minimal hardware

## 🔄 Decisions to Revisit Later

1. **Backend feed fetching** - Add when users want push notifications
2. **Audio caching** - Implement when same articles are converted repeatedly
3. **Cross-device sync** - Add as premium feature
4. **WebSocket for real-time** - Only if adding collaborative features
5. **Microservices** - Stay monolithic until 10,000+ users

## 📚 Technical Documentation

- **API Specification**: `/api/openapi.yaml`
- **Architecture Diagrams**: `/claude/02_architecture.md`
- **TTS Design**: `/claude/03_tts_api_design.md`
- **API Design v2**: `/claude/04_api_design_v2.md`

## 🤝 Team & Contact

- **Solo Developer**: Sergi Gonzalez
- **Project Status**: Pre-launch MVP development
- **Timeline**: Launch Q1 2024
- **Contact**: support@feedtape.app

---

*This document serves as the single source of truth for all technical and product decisions made during FeedTape's development. It should be updated as new decisions are made or existing ones are revisited.*
