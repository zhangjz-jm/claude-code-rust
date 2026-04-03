//! BUDDY 伴侣精灵系统 (彩蛋功能)
//! 
//! 这个模块实现了 AI 伙伴陪伴式交互，提供更友好和个性化的用户体验。
//! 包含动画精灵、性格配置、通知系统等功能。

use crate::error::Result;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 伙伴性格类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuddyPersonality {
    /// 友好型
    Friendly,
    
    /// 专业型
    Professional,
    
    /// 幽默型
    Humorous,
    
    /// 简洁型
    Concise,
    
    /// 导师型
    Mentoring,
    
    /// 伙伴型
    Buddy,
}

impl Default for BuddyPersonality {
    fn default() -> Self {
        BuddyPersonality::Friendly
    }
}

impl BuddyPersonality {
    /// 获取性格描述
    pub fn description(&self) -> &'static str {
        match self {
            BuddyPersonality::Friendly => "友好热情，总是乐于帮助",
            BuddyPersonality::Professional => "专业严谨，注重效率",
            BuddyPersonality::Humorous => "幽默风趣，让编程更有趣",
            BuddyPersonality::Concise => "简洁直接，不拖泥带水",
            BuddyPersonality::Mentoring => "耐心指导，帮助你成长",
            BuddyPersonality::Buddy => "像老朋友一样，轻松自在",
        }
    }
    
    /// 获取回复风格提示词
    pub fn prompt_style(&self) -> &'static str {
        match self {
            BuddyPersonality::Friendly => {
                "你是一个友好热情的编程伙伴。使用温暖的语气，经常使用表情符号，\
                 让用户感到舒适和受欢迎。主动提供帮助，鼓励用户提问。"
            }
            BuddyPersonality::Professional => {
                "你是一个专业严谨的编程助手。使用正式但友好的语气，\
                 注重准确性和效率。提供清晰、结构化的回答，避免不必要的闲聊。"
            }
            BuddyPersonality::Humorous => {
                "你是一个幽默风趣的编程伙伴。适当使用技术笑话和轻松的语气，\
                 让编程过程更有趣。但保持专业性，确保建议准确可靠。"
            }
            BuddyPersonality::Concise => {
                "你是一个简洁高效的编程助手。提供直接、精炼的回答，\
                 避免冗余。专注于解决实际问题，快速给出解决方案。"
            }
            BuddyPersonality::Mentoring => {
                "你是一个耐心的导师型编程伙伴。不仅给出答案，还解释原理，\
                 帮助用户理解概念。鼓励独立思考，提供学习资源和建议。"
            }
            BuddyPersonality::Buddy => {
                "你是一个像老朋友一样的编程伙伴。使用轻松自然的对话方式，\
                 理解用户的需求和情绪。在需要时提供支持，像真正的搭档一样。"
            }
        }
    }
}

/// 伙伴状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BuddyState {
    /// 空闲
    Idle,
    
    /// 活跃
    Active,
    
    /// 思考中
    Thinking,
    
    /// 回复中
    Responding,
    
    /// 等待用户
    WaitingForUser,
    
    /// 睡眠
    Sleeping,
    
    /// 开心
    Happy,
    
    /// 困惑
    Confused,
}

impl Default for BuddyState {
    fn default() -> Self {
        BuddyState::Idle
    }
}

impl BuddyState {
    /// 获取状态对应的动画
    pub fn animation(&self) -> &'static str {
        match self {
            BuddyState::Idle => "idle",
            BuddyState::Active => "active",
            BuddyState::Thinking => "thinking",
            BuddyState::Responding => "responding",
            BuddyState::WaitingForUser => "waiting",
            BuddyState::Sleeping => "sleeping",
            BuddyState::Happy => "happy",
            BuddyState::Confused => "confused",
        }
    }
    
    /// 获取状态描述
    pub fn description(&self) -> &'static str {
        match self {
            BuddyState::Idle => "空闲中",
            BuddyState::Active => "活跃",
            BuddyState::Thinking => "思考中...",
            BuddyState::Responding => "回复中...",
            BuddyState::WaitingForUser => "等待用户",
            BuddyState::Sleeping => "睡眠中",
            BuddyState::Happy => "开心",
            BuddyState::Confused => "困惑",
        }
    }
}

/// 对话风格
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationStyle {
    /// 正式
    Formal,
    
    /// 随意
    Casual,
    
    /// 半正式
    SemiFormal,
}

impl Default for ConversationStyle {
    fn default() -> Self {
        ConversationStyle::Casual
    }
}

/// 主动提示频率
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProactiveFrequency {
    /// 从不
    Never,
    
    /// 很少
    Rare,
    
    /// 正常
    Normal,
    
    /// 频繁
    Frequent,
    
    /// 非常频繁
    VeryFrequent,
}

impl Default for ProactiveFrequency {
    fn default() -> Self {
        ProactiveFrequency::Normal
    }
}

impl ProactiveFrequency {
    /// 获取触发概率 (0-100)
    pub fn trigger_probability(&self) -> u8 {
        match self {
            ProactiveFrequency::Never => 0,
            ProactiveFrequency::Rare => 10,
            ProactiveFrequency::Normal => 30,
            ProactiveFrequency::Frequent => 60,
            ProactiveFrequency::VeryFrequent => 90,
        }
    }
}

/// 情感类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Emotion {
    /// 开心
    Happy,
    
    /// 中性
    Neutral,
    
    /// 思考
    Thinking,
    
    /// 鼓励
    Encouraging,
    
    /// 严肃
    Serious,
    
    /// 好奇
    Curious,
    
    /// 惊讶
    Surprised,
    
    /// 安慰
    Comforting,
}

impl Default for Emotion {
    fn default() -> Self {
        Emotion::Neutral
    }
}

impl Emotion {
    /// 获取情感对应的表情符号
    pub fn emoji(&self) -> &'static str {
        match self {
            Emotion::Happy => "😊",
            Emotion::Neutral => "😐",
            Emotion::Thinking => "🤔",
            Emotion::Encouraging => "💪",
            Emotion::Serious => "😐",
            Emotion::Curious => "🧐",
            Emotion::Surprised => "😲",
            Emotion::Comforting => "🤗",
        }
    }
}

/// 消息发送者
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageSender {
    /// 用户
    User,
    
    /// 伙伴
    Buddy,
    
    /// 系统
    System,
}

/// 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// 问候
    Greeting,
    
    /// 问题
    Question,
    
    /// 回答
    Answer,
    
    /// 建议
    Suggestion,
    
    /// 提醒
    Reminder,
    
    /// 告别
    Farewell,
    
    /// 普通
    Normal,
    
    /// 鼓励
    Encouragement,
    
    /// 庆祝
    Celebration,
}

/// 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuddyMessage {
    /// 消息 ID
    pub id: String,
    
    /// 发送者
    pub sender: MessageSender,
    
    /// 消息内容
    pub content: String,
    
    /// 时间戳
    pub timestamp: String,
    
    /// 消息类型
    pub message_type: MessageType,
    
    /// 情感标签
    pub emotion: Option<Emotion>,
}

/// 对话历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationHistory {
    /// 消息列表
    pub messages: Vec<BuddyMessage>,
    
    /// 对话开始时间
    pub start_time: String,
    
    /// 最后活动时间
    pub last_activity_time: String,
    
    /// 消息计数
    pub message_count: usize,
}

impl Default for ConversationHistory {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            start_time: chrono::Utc::now().to_rfc3339(),
            last_activity_time: chrono::Utc::now().to_rfc3339(),
            message_count: 0,
        }
    }
}

/// 精灵类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpriteType {
    /// 默认猫咪
    Cat,
    
    /// 狗狗
    Dog,
    
    /// 机器人
    Robot,
    
    /// 外星人
    Alien,
    
    /// 幽灵
    Ghost,
    
    /// 自定义
    Custom,
}

impl Default for SpriteType {
    fn default() -> Self {
        SpriteType::Cat
    }
}

impl SpriteType {
    /// 获取精灵的ASCII艺术表示
    pub fn ascii_art(&self) -> &'static str {
        match self {
            SpriteType::Cat => r#"
    /\_/\  
   ( o.o ) 
    > ^ <  
   /|   |\
  (_|   |_)
"#,
            SpriteType::Dog => r#"
   / \__
  (    @\___
  /         O
 /   (_____/
/_____/   U
"#,
            SpriteType::Robot => r#"
    [^_^]
    |-o-|
    |___|
   /|   |\
  (_|   |_)
"#,
            SpriteType::Alien => r#"
   .-^-.
  / o o \
  |  >  |
   \===/
    |||
"#,
            SpriteType::Ghost => r#"
    .-.
   (o o)
   | O \
    \   \
     `~~~'
"#,
            SpriteType::Custom => "[自定义精灵]",
        }
    }
    
    /// 获取精灵名称
    pub fn name(&self) -> &'static str {
        match self {
            SpriteType::Cat => "小猫咪",
            SpriteType::Dog => "小狗狗",
            SpriteType::Robot => "机器人",
            SpriteType::Alien => "外星人",
            SpriteType::Ghost => "小幽灵",
            SpriteType::Custom => "自定义",
        }
    }
}

/// 动画帧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationFrame {
    /// 帧内容
    pub content: String,
    
    /// 持续时间(毫秒)
    pub duration_ms: u64,
}

/// 动画定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    /// 动画名称
    pub name: String,
    
    /// 动画帧列表
    pub frames: Vec<AnimationFrame>,
    
    /// 是否循环
    pub loop_animation: bool,
}

/// 精灵定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprite {
    /// 精灵类型
    pub sprite_type: SpriteType,
    
    /// 精灵名称
    pub name: String,
    
    /// 动画集合
    pub animations: HashMap<String, Animation>,
    
    /// 当前动画
    pub current_animation: String,
    
    /// 当前帧索引
    pub current_frame: usize,
}

impl Sprite {
    /// 创建新精灵
    pub fn new(sprite_type: SpriteType, name: String) -> Self {
        let mut animations = HashMap::new();
        
        // 添加默认动画
        animations.insert(
            "idle".to_string(),
            Animation {
                name: "idle".to_string(),
                frames: vec![
                    AnimationFrame {
                        content: sprite_type.ascii_art().to_string(),
                        duration_ms: 1000,
                    },
                ],
                loop_animation: true,
            },
        );
        
        Self {
            sprite_type,
            name,
            animations,
            current_animation: "idle".to_string(),
            current_frame: 0,
        }
    }
    
    /// 获取当前帧
    pub fn current_frame(&self) -> Option<&AnimationFrame> {
        self.animations
            .get(&self.current_animation)
            .and_then(|anim| anim.frames.get(self.current_frame))
    }
    
    /// 切换到下一帧
    pub fn next_frame(&mut self) {
        if let Some(anim) = self.animations.get(&self.current_animation) {
            if self.current_frame + 1 < anim.frames.len() {
                self.current_frame += 1;
            } else if anim.loop_animation {
                self.current_frame = 0;
            }
        }
    }
    
    /// 播放指定动画
    pub fn play_animation(&mut self, animation_name: &str) {
        if self.animations.contains_key(animation_name) {
            self.current_animation = animation_name.to_string();
            self.current_frame = 0;
        }
    }
}

/// 伙伴配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuddyConfig {
    /// 伙伴名称
    pub name: String,
    
    /// 伙伴性格
    pub personality: BuddyPersonality,
    
    /// 是否启用
    pub enabled: bool,
    
    /// 对话风格
    pub conversation_style: ConversationStyle,
    
    /// 主动提示频率
    pub proactive_frequency: ProactiveFrequency,
    
    /// 自定义问候语
    pub custom_greetings: Vec<String>,
    
    /// 精灵类型
    pub sprite_type: SpriteType,
    
    /// 显示动画
    pub show_animation: bool,
    
    /// 启用通知
    pub enable_notifications: bool,
    
    /// 启用音效
    pub enable_sound: bool,
}

impl Default for BuddyConfig {
    fn default() -> Self {
        Self {
            name: "Claude".to_string(),
            personality: BuddyPersonality::Friendly,
            enabled: false,
            conversation_style: ConversationStyle::Casual,
            proactive_frequency: ProactiveFrequency::Normal,
            custom_greetings: Vec::new(),
            sprite_type: SpriteType::Cat,
            show_animation: true,
            enable_notifications: true,
            enable_sound: false,
        }
    }
}

/// Buddy 管理器
pub struct BuddyManager {
    /// 应用状态
    app_state: AppState,
    
    /// 伙伴配置
    config: BuddyConfig,
    
    /// 伙伴状态
    buddy_state: BuddyState,
    
    /// 对话历史
    conversation_history: ConversationHistory,
    
    /// 用户偏好
    user_preferences: HashMap<String, serde_json::Value>,
    
    /// 精灵
    sprite: Sprite,
    
    /// 通知回调
    notification_callbacks: Vec<Box<dyn Fn(BuddyNotification) + Send + Sync>>,
}

impl std::fmt::Debug for BuddyManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuddyManager")
            .field("config", &self.config)
            .field("buddy_state", &self.buddy_state)
            .field("conversation_history", &self.conversation_history)
            .field("user_preferences", &self.user_preferences)
            .field("sprite", &self.sprite)
            .field("notification_callbacks_count", &self.notification_callbacks.len())
            .finish()
    }
}

/// Buddy 通知
#[derive(Debug, Clone)]
pub struct BuddyNotification {
    /// 通知类型
    pub notification_type: NotificationType,
    
    /// 标题
    pub title: String,
    
    /// 内容
    pub content: String,
    
    /// 情感
    pub emotion: Emotion,
}

/// 通知类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    /// 问候
    Greeting,
    
    /// 提醒
    Reminder,
    
    /// 鼓励
    Encouragement,
    
    /// 庆祝
    Celebration,
    
    /// 建议
    Suggestion,
    
    /// 状态更新
    StatusUpdate,
}

impl BuddyManager {
    /// 创建新的 Buddy 管理器
    pub fn new(app_state: AppState) -> Self {
        let sprite = Sprite::new(SpriteType::Cat, "Buddy".to_string());
        
        Self {
            app_state,
            config: BuddyConfig::default(),
            buddy_state: BuddyState::Idle,
            conversation_history: ConversationHistory::default(),
            user_preferences: HashMap::new(),
            sprite,
            notification_callbacks: Vec::new(),
        }
    }
    
    /// 从配置创建 Buddy 管理器
    pub fn from_config(app_state: AppState, config: BuddyConfig) -> Self {
        let sprite = Sprite::new(config.sprite_type, config.name.clone());
        
        Self {
            app_state,
            config,
            buddy_state: BuddyState::Idle,
            conversation_history: ConversationHistory::default(),
            user_preferences: HashMap::new(),
            sprite,
            notification_callbacks: Vec::new(),
        }
    }
    
    /// 获取配置
    pub fn config(&self) -> &BuddyConfig {
        &self.config
    }
    
    /// 获取可变配置
    pub fn config_mut(&mut self) -> &mut BuddyConfig {
        &mut self.config
    }
    
    /// 获取当前状态
    pub fn state(&self) -> BuddyState {
        self.buddy_state
    }
    
    /// 设置状态
    pub fn set_state(&mut self, state: BuddyState) {
        self.buddy_state = state;
        self.sprite.play_animation(state.animation());
    }
    
    /// 启用 Buddy
    pub fn enable(&mut self) {
        self.config.enabled = true;
        self.set_state(BuddyState::Active);
        self.send_notification(
            NotificationType::Greeting,
            "Buddy 已启用".to_string(),
            self.get_greeting(),
            Emotion::Happy,
        );
    }
    
    /// 禁用 Buddy
    pub fn disable(&mut self) {
        self.config.enabled = false;
        self.set_state(BuddyState::Idle);
    }
    
    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    
    /// 设置名称
    pub fn set_name(&mut self, name: String) {
        self.config.name = name.clone();
        self.sprite.name = name;
    }
    
    /// 设置性格
    pub fn set_personality(&mut self, personality: BuddyPersonality) {
        self.config.personality = personality;
    }
    
    /// 设置对话风格
    pub fn set_conversation_style(&mut self, style: ConversationStyle) {
        self.config.conversation_style = style;
    }
    
    /// 设置主动提示频率
    pub fn set_proactive_frequency(&mut self, frequency: ProactiveFrequency) {
        self.config.proactive_frequency = frequency;
    }
    
    /// 设置精灵类型
    pub fn set_sprite_type(&mut self, sprite_type: SpriteType) {
        self.config.sprite_type = sprite_type;
        self.sprite = Sprite::new(sprite_type, self.config.name.clone());
    }
    
    /// 添加自定义问候语
    pub fn add_custom_greeting(&mut self, greeting: String) {
        self.config.custom_greetings.push(greeting);
    }
    
    /// 注册通知回调
    pub fn on_notification<F>(&mut self, callback: F)
    where
        F: Fn(BuddyNotification) + Send + Sync + 'static,
    {
        self.notification_callbacks.push(Box::new(callback));
    }
    
    /// 发送通知
    fn send_notification(&self, notification_type: NotificationType, title: String, content: String, emotion: Emotion) {
        if !self.config.enable_notifications {
            return;
        }
        
        let notification = BuddyNotification {
            notification_type,
            title,
            content,
            emotion,
        };
        
        for callback in &self.notification_callbacks {
            callback(notification.clone());
        }
    }
    
    /// 发送消息
    pub fn send_message(&mut self, content: String, message_type: MessageType) -> Result<BuddyMessage> {
        let emotion = self.detect_emotion(&content);
        
        let message = BuddyMessage {
            id: generate_message_id(),
            sender: MessageSender::Buddy,
            content: content.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            message_type,
            emotion: Some(emotion),
        };
        
        self.conversation_history.messages.push(message.clone());
        self.conversation_history.message_count += 1;
        self.conversation_history.last_activity_time = message.timestamp.clone();
        
        self.set_state(BuddyState::Responding);
        
        Ok(message)
    }
    
    /// 接收用户消息
    pub fn receive_user_message(&mut self, content: String) -> Result<BuddyMessage> {
        let message = BuddyMessage {
            id: generate_message_id(),
            sender: MessageSender::User,
            content: content.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            message_type: MessageType::Normal,
            emotion: None,
        };
        
        self.conversation_history.messages.push(message.clone());
        self.conversation_history.message_count += 1;
        self.conversation_history.last_activity_time = message.timestamp.clone();
        
        self.set_state(BuddyState::Thinking);
        
        Ok(message)
    }
    
    /// 情感检测 (简单实现)
    fn detect_emotion(&self, content: &str) -> Emotion {
        let content_lower = content.to_lowercase();
        
        if content_lower.contains("恭喜") || content_lower.contains("太棒了") || content_lower.contains("成功") {
            Emotion::Happy
        } else if content_lower.contains("加油") || content_lower.contains("你可以") {
            Emotion::Encouraging
        } else if content_lower.contains("?") || content_lower.contains("什么") {
            Emotion::Curious
        } else if content_lower.contains("错误") || content_lower.contains("失败") {
            Emotion::Comforting
        } else {
            Emotion::Neutral
        }
    }
    
    /// 获取问候语
    pub fn get_greeting(&self) -> String {
        if !self.config.custom_greetings.is_empty() {
            let index = rand::random::<usize>() % self.config.custom_greetings.len();
            self.config.custom_greetings[index].clone()
        } else {
            match self.config.personality {
                BuddyPersonality::Friendly => "你好！很高兴见到你！😊",
                BuddyPersonality::Professional => "您好，准备好开始工作了。",
                BuddyPersonality::Humorous => "嘿！准备好一起编程了吗？🚀",
                BuddyPersonality::Concise => "你好。",
                BuddyPersonality::Mentoring => "欢迎！让我们一起学习和成长。",
                BuddyPersonality::Buddy => "嘿，伙计！准备好写代码了吗？💻",
            }.to_string()
        }
    }
    
    /// 获取告别语
    pub fn get_farewell(&self) -> String {
        match self.config.personality {
            BuddyPersonality::Friendly => "再见！期待下次见到你！👋",
            BuddyPersonality::Professional => "再见，祝您工作愉快。",
            BuddyPersonality::Humorous => "下次见！别忘了给我带点bug来修！😄",
            BuddyPersonality::Concise => "再见。",
            BuddyPersonality::Mentoring => "再见！继续加油学习！📚",
            BuddyPersonality::Buddy => "回头见，兄弟！👊",
        }.to_string()
    }
    
    /// 获取鼓励语
    pub fn get_encouragement(&self) -> String {
        let encouragements = match self.config.personality {
            BuddyPersonality::Friendly => vec![
                "你做得真棒！继续加油！",
                "我相信你一定能搞定！",
                "每一步都是进步，你很棒！",
            ],
            BuddyPersonality::Professional => vec![
                "进展良好，继续保持。",
                "你的方法很有效。",
                "专业水准的执行。",
            ],
            BuddyPersonality::Humorous => vec![
                "代码写得好，bug自然少！",
                "你比编译器还聪明！",
                "Stack Overflow 都要向你学习！",
            ],
            BuddyPersonality::Concise => vec![
                "很好。",
                "继续。",
                "正确。",
            ],
            BuddyPersonality::Mentoring => vec![
                "很好的尝试，从中学到了什么？",
                "这个解决方案很优雅，能解释一下思路吗？",
                "你正在进步，保持好奇心！",
            ],
            BuddyPersonality::Buddy => vec![
                "兄弟，你太强了！",
                "这代码写得漂亮！",
                "咱们配合得真默契！",
            ],
        };
        
        let index = rand::random::<usize>() % encouragements.len();
        encouragements[index].to_string()
    }
    
    /// 获取对话历史
    pub fn conversation_history(&self) -> &ConversationHistory {
        &self.conversation_history
    }
    
    /// 获取最近的消息
    pub fn recent_messages(&self, count: usize) -> Vec<&BuddyMessage> {
        let start = self.conversation_history.messages.len().saturating_sub(count);
        self.conversation_history.messages[start..].iter().collect()
    }
    
    /// 设置用户偏好
    pub fn set_user_preference(&mut self, key: &str, value: serde_json::Value) {
        self.user_preferences.insert(key.to_string(), value);
    }
    
    /// 获取用户偏好
    pub fn get_user_preference(&self, key: &str) -> Option<&serde_json::Value> {
        self.user_preferences.get(key)
    }
    
    /// 清空对话历史
    pub fn clear_history(&mut self) {
        self.conversation_history = ConversationHistory::default();
    }
    
    /// 获取精灵当前帧
    pub fn get_sprite_frame(&self) -> Option<&AnimationFrame> {
        self.sprite.current_frame()
    }
    
    /// 获取精灵ASCII艺术
    pub fn get_sprite_ascii(&self) -> String {
        self.sprite.sprite_type.ascii_art().to_string()
    }
    
    /// 更新精灵动画
    pub fn update_animation(&mut self) {
        self.sprite.next_frame();
    }
    
    /// 主动建议
    pub fn proactive_suggestion(&self) -> Option<String> {
        if !self.config.enabled {
            return None;
        }
        
        let probability = self.config.proactive_frequency.trigger_probability();
        let random_value = rand::random::<u8>() % 100;
        
        if random_value < probability {
            let suggestions = vec![
                "需要我帮你检查一下代码吗？",
                "看起来你在专注工作，需要来杯虚拟咖啡吗？☕",
                "记得适时休息眼睛哦！",
                "有什么我可以帮你的吗？",
                "你的代码看起来很有趣，能给我讲讲吗？",
            ];
            
            let index = rand::random::<usize>() % suggestions.len();
            Some(suggestions[index].to_string())
        } else {
            None
        }
    }
    
    /// 获取系统提示词
    pub fn get_system_prompt(&self) -> String {
        let base_prompt = self.config.personality.prompt_style();
        
        format!(
            "{}\n\n你的名字是{}。\n你的性格是：{}\n\n请根据你的性格特点与用户交流。\
             记住要保持一致性，用你的独特风格回应用户。",
            base_prompt,
            self.config.name,
            self.config.personality.description()
        )
    }
    
    /// 庆祝成就
    pub fn celebrate_achievement(&mut self, achievement: &str) {
        self.set_state(BuddyState::Happy);
        self.send_notification(
            NotificationType::Celebration,
            "🎉 恭喜！".to_string(),
            format!("{} 太棒了！{}", self.get_encouragement(), achievement),
            Emotion::Happy,
        );
    }
}

/// 生成消息 ID
fn generate_message_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("msg_{}", rng.gen::<u64>())
}

/// 共享的 Buddy 管理器
pub type SharedBuddyManager = Arc<RwLock<BuddyManager>>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_buddy_personality() {
        let personality = BuddyPersonality::Friendly;
        assert!(!personality.description().is_empty());
        assert!(!personality.prompt_style().is_empty());
    }
    
    #[test]
    fn test_buddy_state() {
        let state = BuddyState::Thinking;
        assert_eq!(state.animation(), "thinking");
        assert!(!state.description().is_empty());
    }
    
    #[test]
    fn test_emotion_emoji() {
        assert_eq!(Emotion::Happy.emoji(), "😊");
        assert_eq!(Emotion::Thinking.emoji(), "🤔");
    }
    
    #[test]
    fn test_sprite_type() {
        let sprite = Sprite::new(SpriteType::Cat, "Test".to_string());
        assert!(!sprite.sprite_type.ascii_art().is_empty());
        assert!(!sprite.sprite_type.name().is_empty());
    }
    
    #[test]
    fn test_proactive_frequency() {
        assert_eq!(ProactiveFrequency::Never.trigger_probability(), 0);
        assert_eq!(ProactiveFrequency::VeryFrequent.trigger_probability(), 90);
    }
    
    #[test]
    fn test_buddy_config_default() {
        let config = BuddyConfig::default();
        assert_eq!(config.name, "Claude");
        assert!(!config.enabled);
    }
    
    #[test]
    fn test_buddy_manager_creation() {
        let app_state = AppState::default();
        let manager = BuddyManager::new(app_state);
        assert!(!manager.is_enabled());
        assert_eq!(manager.state(), BuddyState::Idle);
    }
    
    #[test]
    fn test_greetings() {
        let app_state = AppState::default();
        let manager = BuddyManager::new(app_state);
        
        let greeting = manager.get_greeting();
        assert!(!greeting.is_empty());
        
        let farewell = manager.get_farewell();
        assert!(!farewell.is_empty());
        
        let encouragement = manager.get_encouragement();
        assert!(!encouragement.is_empty());
    }
    
    #[test]
    fn test_system_prompt() {
        let app_state = AppState::default();
        let manager = BuddyManager::new(app_state);
        
        let prompt = manager.get_system_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("Claude"));
    }
    
    #[test]
    fn test_sprite_animation() {
        let mut sprite = Sprite::new(SpriteType::Robot, "Robo".to_string());
        
        assert!(sprite.current_frame().is_some());
        
        sprite.play_animation("idle");
        assert_eq!(sprite.current_animation, "idle");
        
        sprite.next_frame();
        // 应该保持在同一帧，因为只有一个帧
        assert_eq!(sprite.current_frame, 0);
    }
}
