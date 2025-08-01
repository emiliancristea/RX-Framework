//! Event system for the CX Framework.
//! 
//! This module provides a comprehensive event system that handles user input,
//! window events, and application lifecycle events in a cross-platform manner.

use crate::{Error, Result};
use crate::platform::{PlatformBackend, PlatformEvent, MouseButton, Key, KeyModifiers};
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, VecDeque};

/// Framework-level event enumeration.
/// 
/// These events are processed by the framework and can be handled by
/// applications, widgets, and other components.
#[derive(Debug, Clone)]
pub enum Event {
    /// Application should quit
    Quit,
    
    /// Window events
    WindowClosed { window_id: u64 },
    WindowResized { window_id: u64, width: u32, height: u32 },
    WindowMoved { window_id: u64, x: i32, y: i32 },
    WindowFocused { window_id: u64 },
    WindowUnfocused { window_id: u64 },
    
    /// Mouse events
    MousePressed { window_id: u64, button: MouseButton, x: f64, y: f64 },
    MouseReleased { window_id: u64, button: MouseButton, x: f64, y: f64 },
    MouseMoved { window_id: u64, x: f64, y: f64 },
    MouseEntered { window_id: u64 },
    MouseLeft { window_id: u64 },
    MouseWheel { window_id: u64, delta_x: f64, delta_y: f64 },
    
    /// Keyboard events
    KeyPressed { window_id: u64, key: Key, modifiers: KeyModifiers, repeat: bool },
    KeyReleased { window_id: u64, key: Key, modifiers: KeyModifiers },
    TextInput { window_id: u64, text: String },
    
    /// Custom user events
    User { event_type: String, data: EventData },
}

/// Data payload for custom user events.
#[derive(Debug, Clone)]
pub enum EventData {
    None,
    String(String),
    Number(f64),
    Boolean(bool),
    Custom(HashMap<String, String>),
}

impl Default for EventData {
    fn default() -> Self {
        EventData::None
    }
}

/// Event handler trait for components that can receive events.
pub trait EventHandler: Send + Sync {
    /// Handle an event and return whether it was consumed.
    /// 
    /// If an event is consumed (returns true), it will not be passed
    /// to other handlers in the chain.
    fn handle_event(&mut self, event: &Event) -> Result<bool>;
}

/// Event listener function type.
pub type EventListener = Box<dyn Fn(&Event) -> Result<bool> + Send + Sync>;

/// Event manager that routes events to appropriate handlers.
pub struct EventManager {
    handlers: HashMap<u64, Box<dyn EventHandler>>,
    listeners: Vec<EventListener>,
    next_handler_id: u64,
}

impl EventManager {
    /// Create a new event manager.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            listeners: Vec::new(),
            next_handler_id: 1,
        }
    }
    
    /// Register an event handler and return its ID.
    pub fn register_handler(&mut self, handler: Box<dyn EventHandler>) -> u64 {
        let id = self.next_handler_id;
        self.next_handler_id += 1;
        self.handlers.insert(id, handler);
        id
    }
    
    /// Unregister an event handler by ID.
    pub fn unregister_handler(&mut self, handler_id: u64) -> bool {
        self.handlers.remove(&handler_id).is_some()
    }
    
    /// Add an event listener function.
    pub fn add_listener<F>(&mut self, listener: F)
    where
        F: Fn(&Event) -> Result<bool> + Send + Sync + 'static,
    {
        self.listeners.push(Box::new(listener));
    }
    
    /// Dispatch an event to all registered handlers and listeners.
    pub fn dispatch_event(&mut self, event: &Event) -> Result<()> {
        // First, try listeners
        for listener in &self.listeners {
            if listener(event)? {
                return Ok(()); // Event was consumed
            }
        }
        
        // Then, try handlers
        for handler in self.handlers.values_mut() {
            if handler.handle_event(event)? {
                return Ok(()); // Event was consumed
            }
        }
        
        Ok(())
    }
    
    /// Clear all handlers and listeners.
    pub fn clear(&mut self) {
        self.handlers.clear();
        self.listeners.clear();
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Event loop that processes platform events and converts them to framework events.
pub struct EventLoop {
    backend: Arc<Mutex<Box<dyn PlatformBackend>>>,
    event_queue: VecDeque<Event>,
    event_manager: EventManager,
    mouse_state: MouseState,
    keyboard_state: KeyboardState,
}

#[derive(Debug, Clone)]
pub struct MouseState {
    position: (f64, f64),
    pressed_buttons: Vec<MouseButton>,
    window_id: Option<u64>,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            pressed_buttons: Vec::new(),
            window_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyboardState {
    pressed_keys: HashMap<Key, bool>,
    modifiers: KeyModifiers,
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self {
            pressed_keys: HashMap::new(),
            modifiers: KeyModifiers::default(),
        }
    }
}

impl EventLoop {
    /// Create a new event loop with the given platform backend.
    pub fn new(backend: Arc<Mutex<Box<dyn PlatformBackend>>>) -> Result<Self> {
        Ok(Self {
            backend,
            event_queue: VecDeque::new(),
            event_manager: EventManager::new(),
            mouse_state: MouseState::default(),
            keyboard_state: KeyboardState::default(),
        })
    }
    
    /// Poll for events without blocking.
    pub fn poll_events(&mut self) -> Result<Vec<Event>> {
        let platform_events = {
            let mut backend = self.backend.lock().map_err(|_| Error::event("Failed to lock backend"))?;
            backend.poll_events()?
        };
        
        self.process_platform_events(platform_events)?;
        
        let events: Vec<Event> = self.event_queue.drain(..).collect();
        Ok(events)
    }
    
    /// Wait for events (blocking).
    pub fn wait_events(&mut self) -> Result<Vec<Event>> {
        let platform_events = {
            let mut backend = self.backend.lock().map_err(|_| Error::event("Failed to lock backend"))?;
            backend.wait_events()?
        };
        
        self.process_platform_events(platform_events)?;
        
        let events: Vec<Event> = self.event_queue.drain(..).collect();
        Ok(events)
    }
    
    /// Get a reference to the event manager.
    pub fn event_manager(&mut self) -> &mut EventManager {
        &mut self.event_manager
    }
    
    /// Process platform events and convert them to framework events.
    fn process_platform_events(&mut self, platform_events: Vec<PlatformEvent>) -> Result<()> {
        for platform_event in platform_events {
            match platform_event {
                PlatformEvent::Quit => {
                    self.event_queue.push_back(Event::Quit);
                }
                
                PlatformEvent::WindowClosed { window_id } => {
                    self.event_queue.push_back(Event::WindowClosed { window_id });
                }
                
                PlatformEvent::WindowResized { window_id, width, height } => {
                    self.event_queue.push_back(Event::WindowResized { window_id, width, height });
                }
                
                PlatformEvent::WindowMoved { window_id, x, y } => {
                    self.event_queue.push_back(Event::WindowMoved { window_id, x, y });
                }
                
                PlatformEvent::WindowFocused { window_id } => {
                    self.event_queue.push_back(Event::WindowFocused { window_id });
                }
                
                PlatformEvent::WindowUnfocused { window_id } => {
                    self.event_queue.push_back(Event::WindowUnfocused { window_id });
                }
                
                PlatformEvent::MousePressed { window_id, button, x, y } => {
                    self.mouse_state.position = (x, y);
                    self.mouse_state.window_id = Some(window_id);
                    if !self.mouse_state.pressed_buttons.contains(&button) {
                        self.mouse_state.pressed_buttons.push(button);
                    }
                    
                    self.event_queue.push_back(Event::MousePressed { window_id, button, x, y });
                }
                
                PlatformEvent::MouseReleased { window_id, button, x, y } => {
                    self.mouse_state.position = (x, y);
                    self.mouse_state.pressed_buttons.retain(|&b| b != button);
                    
                    self.event_queue.push_back(Event::MouseReleased { window_id, button, x, y });
                }
                
                PlatformEvent::MouseMoved { window_id, x, y } => {
                    let old_window = self.mouse_state.window_id;
                    self.mouse_state.position = (x, y);
                    
                    // Handle mouse enter/leave events
                    if old_window != Some(window_id) {
                        if let Some(old_id) = old_window {
                            self.event_queue.push_back(Event::MouseLeft { window_id: old_id });
                        }
                        self.event_queue.push_back(Event::MouseEntered { window_id });
                        self.mouse_state.window_id = Some(window_id);
                    }
                    
                    self.event_queue.push_back(Event::MouseMoved { window_id, x, y });
                }
                
                PlatformEvent::MouseWheel { window_id, delta_x, delta_y } => {
                    self.event_queue.push_back(Event::MouseWheel { window_id, delta_x, delta_y });
                }
                
                PlatformEvent::KeyPressed { window_id, key, modifiers } => {
                    let repeat = self.keyboard_state.pressed_keys.get(&key).copied().unwrap_or(false);
                    self.keyboard_state.pressed_keys.insert(key, true);
                    self.keyboard_state.modifiers = modifiers;
                    
                    self.event_queue.push_back(Event::KeyPressed { window_id, key, modifiers, repeat });
                }
                
                PlatformEvent::KeyReleased { window_id, key, modifiers } => {
                    self.keyboard_state.pressed_keys.insert(key, false);
                    self.keyboard_state.modifiers = modifiers;
                    
                    self.event_queue.push_back(Event::KeyReleased { window_id, key, modifiers });
                }
                
                PlatformEvent::TextInput { window_id, text } => {
                    self.event_queue.push_back(Event::TextInput { window_id, text });
                }
            }
        }
        
        Ok(())
    }
    
    /// Get the current mouse state.
    pub fn mouse_state(&self) -> &MouseState {
        &self.mouse_state
    }
    
    /// Get the current keyboard state.
    pub fn keyboard_state(&self) -> &KeyboardState {
        &self.keyboard_state
    }
    
    /// Check if a key is currently pressed.
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.keyboard_state.pressed_keys.get(&key).copied().unwrap_or(false)
    }
    
    /// Check if a mouse button is currently pressed.
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_state.pressed_buttons.contains(&button)
    }
    
    /// Get the current mouse position.
    pub fn mouse_position(&self) -> (f64, f64) {
        self.mouse_state.position
    }
    
    /// Get the current keyboard modifiers.
    pub fn keyboard_modifiers(&self) -> KeyModifiers {
        self.keyboard_state.modifiers
    }
    
    /// Post a custom user event.
    pub fn post_user_event(&mut self, event_type: String, data: EventData) {
        self.event_queue.push_back(Event::User { event_type, data });
    }
}

/// Event filter trait for processing events at different stages.
pub trait EventFilter: Send + Sync {
    /// Filter an event before it's processed.
    /// 
    /// Return `None` to block the event, or `Some(event)` to allow it
    /// (potentially modified).
    fn filter_event(&self, event: Event) -> Option<Event>;
}

/// Event queue for buffering events.
pub struct EventQueue {
    queue: VecDeque<Event>,
    max_size: Option<usize>,
}

impl EventQueue {
    /// Create a new event queue with unlimited size.
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            max_size: None,
        }
    }
    
    /// Create a new event queue with a maximum size.
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(max_size),
            max_size: Some(max_size),
        }
    }
    
    /// Push an event to the queue.
    pub fn push(&mut self, event: Event) {
        if let Some(max) = self.max_size {
            while self.queue.len() >= max {
                self.queue.pop_front();
            }
        }
        self.queue.push_back(event);
    }
    
    /// Pop an event from the queue.
    pub fn pop(&mut self) -> Option<Event> {
        self.queue.pop_front()
    }
    
    /// Get the number of events in the queue.
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    
    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    
    /// Clear all events from the queue.
    pub fn clear(&mut self) {
        self.queue.clear();
    }
    
    /// Drain all events from the queue.
    pub fn drain(&mut self) -> impl Iterator<Item = Event> + '_ {
        self.queue.drain(..)
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_manager() {
        let mut manager = EventManager::new();
        
        // Test listener
        manager.add_listener(move |event| {
            Ok(matches!(event, Event::Quit))
        });
    }

    #[test]
    fn test_event_queue() {
        let mut queue = EventQueue::with_capacity(2);
        
        queue.push(Event::Quit);
        assert_eq!(queue.len(), 1);
        
        queue.push(Event::WindowClosed { window_id: 1 });
        assert_eq!(queue.len(), 2);
        
        // Should remove the first event when capacity is exceeded
        queue.push(Event::WindowClosed { window_id: 2 });
        assert_eq!(queue.len(), 2);
        
        let event = queue.pop().unwrap();
        assert!(matches!(event, Event::WindowClosed { window_id: 1 }));
    }

    #[test]
    fn test_event_data() {
        let data = EventData::String("test".to_string());
        assert!(matches!(data, EventData::String(_)));
        
        let data = EventData::Number(42.0);
        assert!(matches!(data, EventData::Number(_)));
    }
}