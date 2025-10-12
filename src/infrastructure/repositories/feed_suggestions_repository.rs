use crate::domain::feed_suggestions::{Category, FeedSuggestion, FeedSuggestionsRepository};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

static CATEGORIES: LazyLock<Vec<Category>> = LazyLock::new(|| {
    vec![
        Category {
            id: "news-current-affairs".to_string(),
            name: "📰 News & Current Affairs".to_string(),
            description: "Stay informed with breaking news and in-depth analysis from trusted sources around the world".to_string(),
        },
        Category {
            id: "technology-programming".to_string(),
            name: "💻 Technology & Programming".to_string(),
            description: "Latest in tech, programming languages, frameworks, software development, and innovation".to_string(),
        },
        Category {
            id: "science-research".to_string(),
            name: "🔬 Science & Research".to_string(),
            description: "Discoveries, research papers, and breakthrough findings across scientific disciplines".to_string(),
        },
        Category {
            id: "business-finance".to_string(),
            name: "💼 Business & Finance".to_string(),
            description: "Market insights, economic analysis, startup news, and financial intelligence".to_string(),
        },
        Category {
            id: "design-creativity".to_string(),
            name: "🎨 Design & Creativity".to_string(),
            description: "Inspiration for designers, artists, and creative professionals across all mediums".to_string(),
        },
        Category {
            id: "gaming-entertainment".to_string(),
            name: "🎮 Gaming & Entertainment".to_string(),
            description: "Game reviews, industry news, esports, and entertainment content for gamers".to_string(),
        },
        Category {
            id: "health-fitness".to_string(),
            name: "💪 Health & Fitness".to_string(),
            description: "Wellness tips, fitness routines, nutrition advice, and mental health resources".to_string(),
        },
        Category {
            id: "food-cooking".to_string(),
            name: "🍳 Food & Cooking".to_string(),
            description: "Recipes, culinary techniques, restaurant reviews, and food culture from around the world".to_string(),
        },
        Category {
            id: "travel-adventure".to_string(),
            name: "✈️ Travel & Adventure".to_string(),
            description: "Travel guides, destination reviews, adventure stories, and tips for explorers".to_string(),
        },
        Category {
            id: "books-literature".to_string(),
            name: "📚 Books & Literature".to_string(),
            description: "Book reviews, author interviews, literary criticism, and reading recommendations".to_string(),
        },
        Category {
            id: "movies-tv".to_string(),
            name: "🎬 Movies & TV".to_string(),
            description: "Film and television reviews, industry news, and entertainment commentary".to_string(),
        },
        Category {
            id: "music-podcasts".to_string(),
            name: "🎵 Music & Podcasts".to_string(),
            description: "Music news, album reviews, podcast recommendations, and audio content discovery".to_string(),
        },
        Category {
            id: "sports".to_string(),
            name: "⚽ Sports".to_string(),
            description: "Live scores, game analysis, athlete profiles, and sports journalism across all leagues".to_string(),
        },
        Category {
            id: "environment-sustainability".to_string(),
            name: "🌍 Environment & Sustainability".to_string(),
            description: "Climate change, conservation efforts, sustainable living, and environmental news".to_string(),
        },
        Category {
            id: "politics-policy".to_string(),
            name: "🏛️ Politics & Policy".to_string(),
            description: "Political analysis, policy debates, government news, and civic engagement".to_string(),
        },
        Category {
            id: "personal-development".to_string(),
            name: "📈 Personal Development".to_string(),
            description: "Self-improvement, productivity tips, career advice, and personal growth strategies".to_string(),
        },
        Category {
            id: "lifestyle-home".to_string(),
            name: "🏠 Lifestyle & Home".to_string(),
            description: "Home decor, DIY projects, lifestyle trends, and tips for better living".to_string(),
        },
        Category {
            id: "automotive".to_string(),
            name: "🚗 Automotive".to_string(),
            description: "Car reviews, automotive technology, industry news, and vehicle maintenance tips".to_string(),
        },
        Category {
            id: "fashion-beauty".to_string(),
            name: "👗 Fashion & Beauty".to_string(),
            description: "Style trends, beauty tips, fashion industry news, and personal care advice".to_string(),
        },
        Category {
            id: "education-learning".to_string(),
            name: "🎓 Education & Learning".to_string(),
            description: "Educational resources, online courses, teaching strategies, and lifelong learning".to_string(),
        },
    ]
});

static FEED_SUGGESTIONS: LazyLock<Vec<FeedSuggestion>> = LazyLock::new(|| {
    vec![
        // News & Current Affairs (4 feeds)
        FeedSuggestion {
            id: "bbc-news".to_string(),
            title: "BBC News".to_string(),
            description: "Breaking news, analysis and features from the BBC with global coverage and trusted journalism".to_string(),
            url: "https://feeds.bbci.co.uk/news/rss.xml".to_string(),
            category_id: "news-current-affairs".to_string(),
        },
        FeedSuggestion {
            id: "the-guardian".to_string(),
            title: "The Guardian".to_string(),
            description: "Independent journalism covering news, politics, culture, and sport from around the world".to_string(),
            url: "https://www.theguardian.com/rss".to_string(),
            category_id: "news-current-affairs".to_string(),
        },
        FeedSuggestion {
            id: "reuters".to_string(),
            title: "Reuters".to_string(),
            description: "International news and breaking stories from the global news agency trusted by professionals".to_string(),
            url: "https://www.reutersagency.com/feed/".to_string(),
            category_id: "news-current-affairs".to_string(),
        },
        FeedSuggestion {
            id: "npr-news".to_string(),
            title: "NPR News".to_string(),
            description: "National Public Radio's news coverage with in-depth reporting and diverse perspectives".to_string(),
            url: "https://feeds.npr.org/1001/rss.xml".to_string(),
            category_id: "news-current-affairs".to_string(),
        },
        // Technology & Programming (4 feeds)
        FeedSuggestion {
            id: "techcrunch".to_string(),
            title: "TechCrunch".to_string(),
            description: "Breaking technology news, analysis, and opinions from Silicon Valley and beyond with startup focus".to_string(),
            url: "https://techcrunch.com/feed/".to_string(),
            category_id: "technology-programming".to_string(),
        },
        FeedSuggestion {
            id: "hacker-news".to_string(),
            title: "Hacker News".to_string(),
            description: "Social news website focusing on computer science and entrepreneurship from Y Combinator".to_string(),
            url: "https://hnrss.org/frontpage".to_string(),
            category_id: "technology-programming".to_string(),
        },
        FeedSuggestion {
            id: "the-verge".to_string(),
            title: "The Verge".to_string(),
            description: "Technology news, reviews, and analysis with a focus on how tech affects our lives and culture".to_string(),
            url: "https://www.theverge.com/rss/index.xml".to_string(),
            category_id: "technology-programming".to_string(),
        },
        FeedSuggestion {
            id: "dev-to".to_string(),
            title: "DEV Community".to_string(),
            description: "Community of software developers sharing articles, tutorials, and discussions on programming".to_string(),
            url: "https://dev.to/feed".to_string(),
            category_id: "technology-programming".to_string(),
        },
        // Science & Research (4 feeds)
        FeedSuggestion {
            id: "scientific-american".to_string(),
            title: "Scientific American".to_string(),
            description: "Science news and analysis covering research, discoveries, and innovations across all disciplines".to_string(),
            url: "https://www.scientificamerican.com/feed/".to_string(),
            category_id: "science-research".to_string(),
        },
        FeedSuggestion {
            id: "nature-news".to_string(),
            title: "Nature News".to_string(),
            description: "Latest research news from the prestigious international journal covering all sciences".to_string(),
            url: "https://www.nature.com/nature.rss".to_string(),
            category_id: "science-research".to_string(),
        },
        FeedSuggestion {
            id: "science-daily".to_string(),
            title: "ScienceDaily".to_string(),
            description: "Breaking science news and articles on research discoveries from leading universities".to_string(),
            url: "https://www.sciencedaily.com/rss/all.xml".to_string(),
            category_id: "science-research".to_string(),
        },
        FeedSuggestion {
            id: "new-scientist".to_string(),
            title: "New Scientist".to_string(),
            description: "Science news, discoveries, and commentary with focus on making science accessible".to_string(),
            url: "https://www.newscientist.com/feed/home".to_string(),
            category_id: "science-research".to_string(),
        },
        // Business & Finance (4 feeds)
        FeedSuggestion {
            id: "wall-street-journal".to_string(),
            title: "Wall Street Journal".to_string(),
            description: "Business news and financial information with market data, analysis, and economic insights".to_string(),
            url: "https://feeds.a.dj.com/rss/RSSMarketsMain.xml".to_string(),
            category_id: "business-finance".to_string(),
        },
        FeedSuggestion {
            id: "bloomberg".to_string(),
            title: "Bloomberg".to_string(),
            description: "Global business and financial news, stock market updates, and economic analysis".to_string(),
            url: "https://www.bloomberg.com/feed/podcast/bloomberg-intelligence.xml".to_string(),
            category_id: "business-finance".to_string(),
        },
        FeedSuggestion {
            id: "harvard-business-review".to_string(),
            title: "Harvard Business Review".to_string(),
            description: "Management insights, leadership strategies, and business best practices from HBR".to_string(),
            url: "https://hbr.org/feed".to_string(),
            category_id: "business-finance".to_string(),
        },
        FeedSuggestion {
            id: "the-economist".to_string(),
            title: "The Economist".to_string(),
            description: "International news, politics, business, finance, science, and technology analysis".to_string(),
            url: "https://www.economist.com/rss".to_string(),
            category_id: "business-finance".to_string(),
        },
        // Design & Creativity (4 feeds)
        FeedSuggestion {
            id: "smashing-magazine".to_string(),
            title: "Smashing Magazine".to_string(),
            description: "Web design and development articles with focus on UX, UI, and creative coding".to_string(),
            url: "https://www.smashingmagazine.com/feed/".to_string(),
            category_id: "design-creativity".to_string(),
        },
        FeedSuggestion {
            id: "creative-bloq".to_string(),
            title: "Creative Bloq".to_string(),
            description: "Art, design, and creative inspiration covering graphic design, web design, and 3D".to_string(),
            url: "https://www.creativebloq.com/feed".to_string(),
            category_id: "design-creativity".to_string(),
        },
        FeedSuggestion {
            id: "colossal".to_string(),
            title: "Colossal".to_string(),
            description: "Art, design, and visual culture featuring contemporary artists and creative projects".to_string(),
            url: "https://www.thisiscolossal.com/feed/".to_string(),
            category_id: "design-creativity".to_string(),
        },
        FeedSuggestion {
            id: "designboom".to_string(),
            title: "Designboom".to_string(),
            description: "Architecture, design, art, and technology magazine featuring global creative projects".to_string(),
            url: "https://www.designboom.com/feed/".to_string(),
            category_id: "design-creativity".to_string(),
        },
        // Gaming & Entertainment (4 feeds)
        FeedSuggestion {
            id: "ign".to_string(),
            title: "IGN".to_string(),
            description: "Video game news, reviews, previews, and entertainment content for gamers worldwide".to_string(),
            url: "https://feeds.ign.com/ign/all".to_string(),
            category_id: "gaming-entertainment".to_string(),
        },
        FeedSuggestion {
            id: "polygon".to_string(),
            title: "Polygon".to_string(),
            description: "Gaming news, reviews, and features with focus on culture and community".to_string(),
            url: "https://www.polygon.com/rss/index.xml".to_string(),
            category_id: "gaming-entertainment".to_string(),
        },
        FeedSuggestion {
            id: "kotaku".to_string(),
            title: "Kotaku".to_string(),
            description: "Gaming news, reviews, and opinion pieces about video games and gaming culture".to_string(),
            url: "https://kotaku.com/rss".to_string(),
            category_id: "gaming-entertainment".to_string(),
        },
        FeedSuggestion {
            id: "gamespot".to_string(),
            title: "GameSpot".to_string(),
            description: "Comprehensive video game coverage with reviews, news, and gameplay videos".to_string(),
            url: "https://www.gamespot.com/feeds/mashup/".to_string(),
            category_id: "gaming-entertainment".to_string(),
        },
        // Health & Fitness (4 feeds)
        FeedSuggestion {
            id: "healthline".to_string(),
            title: "Healthline".to_string(),
            description: "Evidence-based health and wellness information with medical review and expert advice".to_string(),
            url: "https://www.healthline.com/rss".to_string(),
            category_id: "health-fitness".to_string(),
        },
        FeedSuggestion {
            id: "mens-health".to_string(),
            title: "Men's Health".to_string(),
            description: "Fitness, nutrition, style, and health tips for men seeking to improve their wellbeing".to_string(),
            url: "https://www.menshealth.com/rss/all.xml/".to_string(),
            category_id: "health-fitness".to_string(),
        },
        FeedSuggestion {
            id: "womens-health".to_string(),
            title: "Women's Health".to_string(),
            description: "Health, fitness, nutrition, and wellness content specifically for women".to_string(),
            url: "https://www.womenshealthmag.com/rss/all.xml/".to_string(),
            category_id: "health-fitness".to_string(),
        },
        FeedSuggestion {
            id: "yoga-journal".to_string(),
            title: "Yoga Journal".to_string(),
            description: "Yoga practices, mindfulness, meditation, and holistic wellness guidance".to_string(),
            url: "https://www.yogajournal.com/feed/".to_string(),
            category_id: "health-fitness".to_string(),
        },
        // Food & Cooking (4 feeds)
        FeedSuggestion {
            id: "serious-eats".to_string(),
            title: "Serious Eats".to_string(),
            description: "Recipes, cooking techniques, and food science for passionate home cooks".to_string(),
            url: "https://www.seriouseats.com/feed".to_string(),
            category_id: "food-cooking".to_string(),
        },
        FeedSuggestion {
            id: "food52".to_string(),
            title: "Food52".to_string(),
            description: "Community-driven recipes, cooking tips, and food stories from home cooks".to_string(),
            url: "https://food52.com/blog.rss".to_string(),
            category_id: "food-cooking".to_string(),
        },
        FeedSuggestion {
            id: "bon-appetit".to_string(),
            title: "Bon Appétit".to_string(),
            description: "Recipes, restaurant reviews, and food trends from the iconic culinary magazine".to_string(),
            url: "https://www.bonappetit.com/feed/rss".to_string(),
            category_id: "food-cooking".to_string(),
        },
        FeedSuggestion {
            id: "the-kitchn".to_string(),
            title: "The Kitchn".to_string(),
            description: "Cooking inspiration, kitchen tips, and recipes for everyday meals and special occasions".to_string(),
            url: "https://www.thekitchn.com/main.rss".to_string(),
            category_id: "food-cooking".to_string(),
        },
        // Travel & Adventure (4 feeds)
        FeedSuggestion {
            id: "lonely-planet".to_string(),
            title: "Lonely Planet".to_string(),
            description: "Travel guides, destination inspiration, and tips from the world's leading travel authority".to_string(),
            url: "https://www.lonelyplanet.com/feeds/blog/rss".to_string(),
            category_id: "travel-adventure".to_string(),
        },
        FeedSuggestion {
            id: "national-geographic-travel".to_string(),
            title: "National Geographic Travel".to_string(),
            description: "Stunning photography, travel stories, and cultural insights from around the globe".to_string(),
            url: "https://www.nationalgeographic.com/travel/rss".to_string(),
            category_id: "travel-adventure".to_string(),
        },
        FeedSuggestion {
            id: "conde-nast-traveler".to_string(),
            title: "Condé Nast Traveler".to_string(),
            description: "Luxury travel guides, hotel reviews, and destination recommendations".to_string(),
            url: "https://www.cntraveler.com/feed/rss".to_string(),
            category_id: "travel-adventure".to_string(),
        },
        FeedSuggestion {
            id: "nomadic-matt".to_string(),
            title: "Nomadic Matt".to_string(),
            description: "Budget travel tips, destination guides, and money-saving strategies for travelers".to_string(),
            url: "https://www.nomadicmatt.com/feed/".to_string(),
            category_id: "travel-adventure".to_string(),
        },
        // Books & Literature (4 feeds)
        FeedSuggestion {
            id: "literary-hub".to_string(),
            title: "Literary Hub".to_string(),
            description: "Book news, author interviews, essays, and literary criticism from leading voices".to_string(),
            url: "https://lithub.com/feed/".to_string(),
            category_id: "books-literature".to_string(),
        },
        FeedSuggestion {
            id: "book-riot".to_string(),
            title: "Book Riot".to_string(),
            description: "Book recommendations, reading lists, and literary news for passionate readers".to_string(),
            url: "https://bookriot.com/feed/".to_string(),
            category_id: "books-literature".to_string(),
        },
        FeedSuggestion {
            id: "ny-times-books".to_string(),
            title: "NY Times Books".to_string(),
            description: "Book reviews, bestseller lists, and literary coverage from The New York Times".to_string(),
            url: "https://rss.nytimes.com/services/xml/rss/nyt/Books.xml".to_string(),
            category_id: "books-literature".to_string(),
        },
        FeedSuggestion {
            id: "goodreads-blog".to_string(),
            title: "Goodreads Blog".to_string(),
            description: "Book recommendations, author interviews, and reading lists from the Goodreads community".to_string(),
            url: "https://www.goodreads.com/blog.xml".to_string(),
            category_id: "books-literature".to_string(),
        },
        // Movies & TV (4 feeds)
        FeedSuggestion {
            id: "variety".to_string(),
            title: "Variety".to_string(),
            description: "Entertainment industry news covering film, television, and streaming content".to_string(),
            url: "https://variety.com/feed/".to_string(),
            category_id: "movies-tv".to_string(),
        },
        FeedSuggestion {
            id: "hollywood-reporter".to_string(),
            title: "Hollywood Reporter".to_string(),
            description: "Breaking entertainment news, film reviews, and Hollywood insider coverage".to_string(),
            url: "https://www.hollywoodreporter.com/feed/".to_string(),
            category_id: "movies-tv".to_string(),
        },
        FeedSuggestion {
            id: "indiewire".to_string(),
            title: "IndieWire".to_string(),
            description: "Film and television news with focus on independent and arthouse cinema".to_string(),
            url: "https://www.indiewire.com/feed/".to_string(),
            category_id: "movies-tv".to_string(),
        },
        FeedSuggestion {
            id: "rotten-tomatoes".to_string(),
            title: "Rotten Tomatoes".to_string(),
            description: "Movie and TV reviews aggregated from critics with audience ratings and recommendations".to_string(),
            url: "https://editorial.rottentomatoes.com/feed/".to_string(),
            category_id: "movies-tv".to_string(),
        },
        // Music & Podcasts (4 feeds)
        FeedSuggestion {
            id: "pitchfork".to_string(),
            title: "Pitchfork".to_string(),
            description: "Music news, album reviews, and features covering indie, rock, rap, and electronic music".to_string(),
            url: "https://pitchfork.com/rss/news/".to_string(),
            category_id: "music-podcasts".to_string(),
        },
        FeedSuggestion {
            id: "rolling-stone".to_string(),
            title: "Rolling Stone".to_string(),
            description: "Music news, album reviews, and cultural commentary from the iconic music magazine".to_string(),
            url: "https://www.rollingstone.com/feed/".to_string(),
            category_id: "music-podcasts".to_string(),
        },
        FeedSuggestion {
            id: "consequence".to_string(),
            title: "Consequence".to_string(),
            description: "Music, film, and TV news with album reviews and entertainment coverage".to_string(),
            url: "https://consequence.net/feed/".to_string(),
            category_id: "music-podcasts".to_string(),
        },
        FeedSuggestion {
            id: "stereogum".to_string(),
            title: "Stereogum".to_string(),
            description: "Indie music blog with news, reviews, and MP3s covering rock and alternative".to_string(),
            url: "https://www.stereogum.com/feed/".to_string(),
            category_id: "music-podcasts".to_string(),
        },
        // Sports (4 feeds)
        FeedSuggestion {
            id: "espn".to_string(),
            title: "ESPN".to_string(),
            description: "Comprehensive sports coverage including scores, news, and analysis across all leagues".to_string(),
            url: "https://www.espn.com/espn/rss/news".to_string(),
            category_id: "sports".to_string(),
        },
        FeedSuggestion {
            id: "the-athletic".to_string(),
            title: "The Athletic".to_string(),
            description: "In-depth sports journalism with beat writers covering teams and leagues".to_string(),
            url: "https://theathletic.com/rss/".to_string(),
            category_id: "sports".to_string(),
        },
        FeedSuggestion {
            id: "bleacher-report".to_string(),
            title: "Bleacher Report".to_string(),
            description: "Sports news, highlights, and fan-focused coverage of major sports leagues".to_string(),
            url: "https://bleacherreport.com/articles/feed".to_string(),
            category_id: "sports".to_string(),
        },
        FeedSuggestion {
            id: "sports-illustrated".to_string(),
            title: "Sports Illustrated".to_string(),
            description: "Sports journalism featuring long-form stories, analysis, and iconic photography".to_string(),
            url: "https://www.si.com/rss/si_topstories.rss".to_string(),
            category_id: "sports".to_string(),
        },
        // Environment & Sustainability (4 feeds)
        FeedSuggestion {
            id: "grist".to_string(),
            title: "Grist".to_string(),
            description: "Climate change news and environmental journalism with solutions-focused reporting".to_string(),
            url: "https://grist.org/feed/".to_string(),
            category_id: "environment-sustainability".to_string(),
        },
        FeedSuggestion {
            id: "treehugger".to_string(),
            title: "Treehugger".to_string(),
            description: "Sustainability news covering green living, renewable energy, and environmental issues".to_string(),
            url: "https://www.treehugger.com/feeds".to_string(),
            category_id: "environment-sustainability".to_string(),
        },
        FeedSuggestion {
            id: "yale-environment-360".to_string(),
            title: "Yale Environment 360".to_string(),
            description: "Environmental news and analysis from Yale School of the Environment".to_string(),
            url: "https://e360.yale.edu/feed".to_string(),
            category_id: "environment-sustainability".to_string(),
        },
        FeedSuggestion {
            id: "climate-central".to_string(),
            title: "Climate Central".to_string(),
            description: "Climate science research and journalism making climate change understandable".to_string(),
            url: "https://www.climatecentral.org/feed".to_string(),
            category_id: "environment-sustainability".to_string(),
        },
        // Politics & Policy (4 feeds)
        FeedSuggestion {
            id: "politico".to_string(),
            title: "Politico".to_string(),
            description: "Political news, policy analysis, and insider coverage of Washington and beyond".to_string(),
            url: "https://www.politico.com/rss/politicopicks.xml".to_string(),
            category_id: "politics-policy".to_string(),
        },
        FeedSuggestion {
            id: "the-hill".to_string(),
            title: "The Hill".to_string(),
            description: "Political news covering Congress, campaigns, and the White House with analysis".to_string(),
            url: "https://thehill.com/feed/".to_string(),
            category_id: "politics-policy".to_string(),
        },
        FeedSuggestion {
            id: "foreign-policy".to_string(),
            title: "Foreign Policy".to_string(),
            description: "International relations, global politics, and foreign affairs analysis".to_string(),
            url: "https://foreignpolicy.com/feed/".to_string(),
            category_id: "politics-policy".to_string(),
        },
        FeedSuggestion {
            id: "politifact".to_string(),
            title: "PolitiFact".to_string(),
            description: "Fact-checking political claims with Pulitzer Prize-winning journalism".to_string(),
            url: "https://www.politifact.com/rss/all/".to_string(),
            category_id: "politics-policy".to_string(),
        },
        // Personal Development (4 feeds)
        FeedSuggestion {
            id: "zen-habits".to_string(),
            title: "Zen Habits".to_string(),
            description: "Mindfulness, simplicity, and productivity tips for a more focused life".to_string(),
            url: "https://zenhabits.net/feed/".to_string(),
            category_id: "personal-development".to_string(),
        },
        FeedSuggestion {
            id: "lifehacker".to_string(),
            title: "Lifehacker".to_string(),
            description: "Productivity tips, life hacks, and software recommendations for better living".to_string(),
            url: "https://lifehacker.com/rss".to_string(),
            category_id: "personal-development".to_string(),
        },
        FeedSuggestion {
            id: "tiny-buddha".to_string(),
            title: "Tiny Buddha".to_string(),
            description: "Simple wisdom for complex lives with mindfulness and personal growth insights".to_string(),
            url: "https://tinybuddha.com/feed/".to_string(),
            category_id: "personal-development".to_string(),
        },
        FeedSuggestion {
            id: "james-clear".to_string(),
            title: "James Clear".to_string(),
            description: "Habits, decision making, and continuous improvement from the Atomic Habits author".to_string(),
            url: "https://jamesclear.com/feed".to_string(),
            category_id: "personal-development".to_string(),
        },
        // Lifestyle & Home (4 feeds)
        FeedSuggestion {
            id: "apartment-therapy".to_string(),
            title: "Apartment Therapy".to_string(),
            description: "Home decor inspiration, DIY projects, and apartment living tips".to_string(),
            url: "https://www.apartmenttherapy.com/main.rss".to_string(),
            category_id: "lifestyle-home".to_string(),
        },
        FeedSuggestion {
            id: "design-sponge".to_string(),
            title: "Design*Sponge".to_string(),
            description: "Interior design ideas, home tours, and DIY projects for creative living".to_string(),
            url: "https://www.designsponge.com/feed".to_string(),
            category_id: "lifestyle-home".to_string(),
        },
        FeedSuggestion {
            id: "remodelista".to_string(),
            title: "Remodelista".to_string(),
            description: "Design inspiration for home renovation, remodeling, and interior design".to_string(),
            url: "https://www.remodelista.com/posts/feed/".to_string(),
            category_id: "lifestyle-home".to_string(),
        },
        FeedSuggestion {
            id: "real-simple".to_string(),
            title: "Real Simple".to_string(),
            description: "Practical solutions for everyday life with organizing tips and home management".to_string(),
            url: "https://www.realsimple.com/syndication/all".to_string(),
            category_id: "lifestyle-home".to_string(),
        },
        // Automotive (4 feeds)
        FeedSuggestion {
            id: "car-and-driver".to_string(),
            title: "Car and Driver".to_string(),
            description: "Automotive reviews, road tests, and car buying advice from industry experts".to_string(),
            url: "https://www.caranddriver.com/rss/all.xml/".to_string(),
            category_id: "automotive".to_string(),
        },
        FeedSuggestion {
            id: "motor-trend".to_string(),
            title: "MotorTrend".to_string(),
            description: "Car reviews, automotive news, and vehicle comparisons for enthusiasts".to_string(),
            url: "https://www.motortrend.com/feed/".to_string(),
            category_id: "automotive".to_string(),
        },
        FeedSuggestion {
            id: "jalopnik".to_string(),
            title: "Jalopnik".to_string(),
            description: "Car news, reviews, and automotive culture for passionate car enthusiasts".to_string(),
            url: "https://jalopnik.com/rss".to_string(),
            category_id: "automotive".to_string(),
        },
        FeedSuggestion {
            id: "autoblog".to_string(),
            title: "Autoblog".to_string(),
            description: "Automotive news, reviews, and advice covering cars, trucks, and EVs".to_string(),
            url: "https://www.autoblog.com/rss.xml".to_string(),
            category_id: "automotive".to_string(),
        },
        // Fashion & Beauty (4 feeds)
        FeedSuggestion {
            id: "vogue".to_string(),
            title: "Vogue".to_string(),
            description: "Fashion news, runway coverage, and beauty trends from the iconic style authority".to_string(),
            url: "https://www.vogue.com/feed/rss".to_string(),
            category_id: "fashion-beauty".to_string(),
        },
        FeedSuggestion {
            id: "elle".to_string(),
            title: "Elle".to_string(),
            description: "Fashion trends, beauty tips, and style advice from the international magazine".to_string(),
            url: "https://www.elle.com/rss/all.xml/".to_string(),
            category_id: "fashion-beauty".to_string(),
        },
        FeedSuggestion {
            id: "fashionista".to_string(),
            title: "Fashionista".to_string(),
            description: "Fashion industry news, trends, and career advice for fashion professionals".to_string(),
            url: "https://fashionista.com/feed".to_string(),
            category_id: "fashion-beauty".to_string(),
        },
        FeedSuggestion {
            id: "into-the-gloss".to_string(),
            title: "Into The Gloss".to_string(),
            description: "Beauty tips, product recommendations, and skincare advice from beauty insiders".to_string(),
            url: "https://intothegloss.com/feed/".to_string(),
            category_id: "fashion-beauty".to_string(),
        },
        // Education & Learning (4 feeds)
        FeedSuggestion {
            id: "edutopia".to_string(),
            title: "Edutopia".to_string(),
            description: "Teaching strategies, education technology, and classroom innovation from George Lucas".to_string(),
            url: "https://www.edutopia.org/rss.xml".to_string(),
            category_id: "education-learning".to_string(),
        },
        FeedSuggestion {
            id: "edsurge".to_string(),
            title: "EdSurge".to_string(),
            description: "Education technology news covering edtech tools, online learning, and innovation".to_string(),
            url: "https://www.edsurge.com/rss".to_string(),
            category_id: "education-learning".to_string(),
        },
        FeedSuggestion {
            id: "chronicle-higher-education".to_string(),
            title: "Chronicle of Higher Education".to_string(),
            description: "News and analysis about colleges, universities, and academic life".to_string(),
            url: "https://www.chronicle.com/rss".to_string(),
            category_id: "education-learning".to_string(),
        },
        FeedSuggestion {
            id: "khan-academy-blog".to_string(),
            title: "Khan Academy Blog".to_string(),
            description: "Free educational resources, learning strategies, and success stories from Khan Academy".to_string(),
            url: "https://blog.khanacademy.org/feed/".to_string(),
            category_id: "education-learning".to_string(),
        },
    ]
});

pub struct HardcodedFeedSuggestionsRepository;

impl HardcodedFeedSuggestionsRepository {
    pub fn new() -> Self {
        // Verify data integrity at construction time
        debug_assert_eq!(CATEGORIES.len(), 20, "Must have exactly 20 categories");
        debug_assert_eq!(
            FEED_SUGGESTIONS.len(),
            80,
            "Must have exactly 80 suggestions"
        );

        // Verify each category has exactly 4 suggestions
        let mut counts: HashMap<&String, usize> = HashMap::new();
        for suggestion in FEED_SUGGESTIONS.iter() {
            *counts.entry(&suggestion.category_id).or_insert(0) += 1;
        }

        for category in CATEGORIES.iter() {
            debug_assert_eq!(
                counts.get(&category.id),
                Some(&4),
                "Category {} must have exactly 4 suggestions",
                category.id
            );
        }

        Self
    }
}

impl FeedSuggestionsRepository for HardcodedFeedSuggestionsRepository {
    fn get_all_categories(&self) -> Vec<Category> {
        let mut categories = CATEGORIES.clone();
        // Sort alphabetically by name
        categories.sort_by(|a, b| a.name.cmp(&b.name));
        categories
    }

    fn get_suggestions_by_categories(&self, category_ids: &[String]) -> Vec<FeedSuggestion> {
        let valid_category_ids: HashSet<&String> = CATEGORIES.iter().map(|c| &c.id).collect();

        // Log warnings for invalid category IDs
        for id in category_ids {
            if !valid_category_ids.contains(id) {
                tracing::warn!(category_id = %id, "Invalid category ID requested");
            }
        }

        // Filter suggestions and deduplicate by URL
        let mut seen_urls: HashSet<&String> = HashSet::new();
        let mut results = Vec::new();

        for suggestion in FEED_SUGGESTIONS.iter() {
            if category_ids.contains(&suggestion.category_id) && seen_urls.insert(&suggestion.url) {
                results.push(suggestion.clone());
            }
        }

        results
    }
}

impl Default for HardcodedFeedSuggestionsRepository {
    fn default() -> Self {
        Self::new()
    }
}
