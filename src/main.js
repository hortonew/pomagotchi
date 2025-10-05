const { invoke } = window.__TAURI__.core;

// Timer state
let timerState = {
    minutes: 25,
    seconds: 0,
    isRunning: false,
    isPaused: false,
    intervalId: null,
    initialTotalSeconds: 1500 // Track initial duration in seconds for XP calculation
};

// Last selected timer duration for restoration after completion
let lastSelectedDuration = {
    minutes: 25,
    seconds: 0
};

// Creature state
let creatureState = {
    level: 1,
    xp: 0,
    xpNeeded: 100,
    stage: 'egg' // egg, baby, teen, adult
};

// DOM elements
let timerDisplay;
let startBtn;
let pauseBtn;
let resetBtn;
let completeBtn;
let creatureDisplay;
let levelDisplay;
let xpDisplay;
let xpNeededDisplay;
let xpProgress;
let notificationContainer;
let totalPomodorosDisplay;
let totalXpDisplay;
let currentStreakDisplay;
let studyTimeDisplay;

// Backup storage for undo functionality
let dataBackup = null;

// Notification configuration
const notificationConfig = {
    success: { duration: 3000, showCountdown: false },
    evolution: { duration: 5000, showCountdown: false },
    xp: { duration: 3000, showCountdown: false },
    warning: { duration: 8000, showCountdown: true },
    error: { duration: 4000, showCountdown: false },
    withAction: { duration: 8000, showCountdown: true }
};

// Enhanced notification system with action button and countdown
function showNotificationWithAction(message, type = 'success', actionText = null, actionCallback = null) {
    const notification = document.createElement('div');
    notification.className = `notification ${type}`;

    let icon = '🎉';
    if (type === 'evolution') {
        icon = '🌟';
    } else if (type === 'xp') {
        icon = '✨';
    } else if (type === 'warning') {
        icon = '⚠️';
    } else if (type === 'error') {
        icon = '❌';
    }

    // Get config for this notification type
    const config = actionText ? notificationConfig.withAction : notificationConfig[type] || notificationConfig.success;
    const duration = config.duration;
    const showCountdown = config.showCountdown || actionText !== null;

    notification.innerHTML = `
        <span class="notification-icon">${icon}</span>
        <span class="notification-message">${message}</span>
        ${showCountdown ? '<span class="notification-countdown"></span>' : ''}
    `;

    // Add action button if provided
    if (actionText && actionCallback) {
        const actionButton = document.createElement('button');
        actionButton.className = 'notification-action';
        actionButton.textContent = actionText;
        actionButton.onclick = () => {
            // Clear countdown if active
            if (notification.countdownInterval) {
                clearInterval(notification.countdownInterval);
            }
            // Remove the notification
            notification.classList.remove('show');
            setTimeout(() => {
                if (notificationContainer.contains(notification)) {
                    notificationContainer.removeChild(notification);
                }
            }, 300);

            // Execute the callback
            actionCallback();
        };
        notification.appendChild(actionButton);
    }

    notificationContainer.appendChild(notification);

    // Trigger animation
    setTimeout(() => {
        notification.classList.add('show');
    }, 10);

    // Setup countdown if enabled
    if (showCountdown) {
        const countdownElement = notification.querySelector('.notification-countdown');
        if (countdownElement) {
            let timeLeft = duration / 1000; // Convert to seconds

            const updateCountdown = () => {
                countdownElement.textContent = `${timeLeft.toFixed(1)}s`;
                timeLeft -= 0.1;

                if (timeLeft <= 0) {
                    clearInterval(notification.countdownInterval);
                }
            };

            // Initial update
            updateCountdown();

            // Update every 100ms for smooth countdown
            notification.countdownInterval = setInterval(updateCountdown, 100);
        }
    }

    // Remove notification after delay
    setTimeout(() => {
        if (notification.countdownInterval) {
            clearInterval(notification.countdownInterval);
        }
        notification.classList.remove('show');
        setTimeout(() => {
            if (notificationContainer.contains(notification)) {
                notificationContainer.removeChild(notification);
            }
        }, 300);
    }, duration);

    return notification;
}

// Notification system
function showNotification(message, type = 'success') {
    return showNotificationWithAction(message, type);
}

// Function to configure notification durations
function configureNotifications(config) {
    Object.assign(notificationConfig, config);
}

// Make notification config accessible globally for customization
window.configureNotifications = configureNotifications;
window.notificationConfig = notificationConfig;

// Initialize app when DOM is loaded
window.addEventListener("DOMContentLoaded", () => {
    initializeElements();
    setupEventListeners();
    updateTimerDisplay();
    updateCreatureDisplay();
    updatePresetButtonStates(25, 0); // Set 25m as default active
    loadFullGameState();
});

function initializeElements() {
    timerDisplay = document.getElementById('timer-display');
    startBtn = document.getElementById('start-btn');
    pauseBtn = document.getElementById('pause-btn');
    resetBtn = document.getElementById('reset-btn');
    completeBtn = document.getElementById('complete-btn');
    creatureDisplay = document.getElementById('creature-display');
    levelDisplay = document.getElementById('creature-level');
    xpDisplay = document.getElementById('creature-xp');
    xpNeededDisplay = document.getElementById('xp-needed');
    xpProgress = document.getElementById('xp-progress');
    notificationContainer = document.getElementById('notification');
    totalPomodorosDisplay = document.getElementById('total-pomodoros');
    totalXpDisplay = document.getElementById('total-xp');
    currentStreakDisplay = document.getElementById('current-streak');
    studyTimeDisplay = document.getElementById('study-time');
}

function setupEventListeners() {
    // Preset buttons
    document.querySelectorAll('.preset-button').forEach(btn => {
        btn.addEventListener('click', (e) => {
            const minutes = parseInt(e.target.dataset.minutes) || 0;
            const seconds = parseInt(e.target.dataset.seconds) || 0;
            setTimer(minutes, seconds);
        });
    });

    // Control buttons
    startBtn.addEventListener('click', startTimer);
    pauseBtn.addEventListener('click', pauseTimer);
    resetBtn.addEventListener('click', resetTimer);
    completeBtn.addEventListener('click', completeTimerNow);
}

function setTimer(minutes, seconds = 0) {
    if (!timerState.isRunning) {
        timerState.minutes = minutes;
        timerState.seconds = seconds;

        // Set initial duration for XP calculation
        timerState.initialTotalSeconds = (minutes * 60) + seconds;

        // Remember this selection for restoration after completion
        lastSelectedDuration.minutes = minutes;
        lastSelectedDuration.seconds = seconds;

        updateTimerDisplay();
        updatePresetButtonStates(minutes, seconds);
    }
}

function updatePresetButtonStates(activeMinutes, activeSeconds = 0) {
    document.querySelectorAll('.preset-button').forEach(btn => {
        const btnMinutes = parseInt(btn.dataset.minutes) || 0;
        const btnSeconds = parseInt(btn.dataset.seconds) || 0;

        if (btnMinutes === activeMinutes && btnSeconds === activeSeconds) {
            btn.classList.add('active');
        } else {
            btn.classList.remove('active');
        }
    });
}

function startTimer() {
    if (!timerState.isRunning) {
        timerState.isRunning = true;
        timerState.isPaused = false;

        timerState.intervalId = setInterval(() => {
            if (timerState.seconds === 0) {
                if (timerState.minutes === 0) {
                    // Timer completed naturally
                    completePomodoro(true);
                    return;
                }
                timerState.minutes--;
                timerState.seconds = 59;
            } else {
                timerState.seconds--;
            }
            updateTimerDisplay();
        }, 1000);

        updateButtonStates();
    }
}

function pauseTimer() {
    if (timerState.isRunning) {
        timerState.isRunning = false;
        timerState.isPaused = true;
        clearInterval(timerState.intervalId);
        updateButtonStates();
        saveTimerState();
    }
}

function resetTimer() {
    timerState.isRunning = false;
    timerState.isPaused = false;
    clearInterval(timerState.intervalId);

    // Reset to the last selected duration
    timerState.minutes = lastSelectedDuration.minutes;
    timerState.seconds = lastSelectedDuration.seconds;
    timerState.initialTotalSeconds = (lastSelectedDuration.minutes * 60) + lastSelectedDuration.seconds;
    updateTimerDisplay();
    updateButtonStates();
    updatePresetButtonStates(lastSelectedDuration.minutes, lastSelectedDuration.seconds);
    saveTimerState();
}

function completeTimerNow() {
    if (timerState.isRunning || timerState.isPaused) {
        completePomodoro(false); // false indicates early completion
    }
}

function calculateXP(isNaturalCompletion = true) {
    if (isNaturalCompletion) {
        return 25; // Full XP for natural completion
    }

    // Calculate remaining time and percentage completed
    const remainingSeconds = (timerState.minutes * 60) + timerState.seconds;
    const completedSeconds = timerState.initialTotalSeconds - remainingSeconds;
    const completionPercentage = completedSeconds / timerState.initialTotalSeconds;

    // Calculate XP as percentage of 25, rounded up
    const partialXP = Math.ceil(25 * completionPercentage);

    // Ensure minimum of 1 XP for any effort
    return Math.max(1, partialXP);
}

function completePomodoro(isNaturalCompletion = true) {
    timerState.isRunning = false;
    clearInterval(timerState.intervalId);

    // Calculate XP and duration for backend
    const xpGained = calculateXP(isNaturalCompletion);
    const durationSeconds = timerState.initialTotalSeconds - ((timerState.minutes * 60) + timerState.seconds);

    // Update local creature state immediately for UI responsiveness
    creatureState.xp += xpGained;
    checkEvolution();

    // Send completion data to backend
    completeSessionOnBackend(durationSeconds, xpGained);

    // Reset timer to the last selected duration
    timerState.minutes = lastSelectedDuration.minutes;
    timerState.seconds = lastSelectedDuration.seconds;
    timerState.initialTotalSeconds = (lastSelectedDuration.minutes * 60) + lastSelectedDuration.seconds;
    updateTimerDisplay();
    updateButtonStates();
    updatePresetButtonStates(lastSelectedDuration.minutes, lastSelectedDuration.seconds);
    updateCreatureDisplay();
    saveTimerState();

    // Show completion message
    const completionType = isNaturalCompletion ? 'completed' : 'completed early';
    showNotification(`Pomodoro ${completionType}! Your creature gained ${xpGained} XP!`, 'xp');
}

async function completeSessionOnBackend(durationSeconds, xpGained) {
    try {
        const progress = await invoke('complete_pomodoro', {
            durationSeconds,
            xpGained
        });

        // Update progress display with latest data
        updateProgressDisplay(progress);

        console.log('Session completed:', progress);
    } catch (error) {
        console.error('Failed to complete session on backend:', error);
    }
}

function checkEvolution() {
    let evolved = false;
    while (creatureState.xp >= creatureState.xpNeeded) {
        creatureState.level++;
        creatureState.xp -= creatureState.xpNeeded;
        creatureState.xpNeeded = Math.floor(creatureState.xpNeeded * 1.5);

        // Update creature stage
        if (creatureState.level === 2) {
            creatureState.stage = 'baby';
        } else if (creatureState.level === 3) {
            creatureState.stage = 'teen';
        } else if (creatureState.level >= 4) {
            creatureState.stage = 'adult';
        }

        evolved = true;
    }

    if (evolved) {
        saveCreatureState();
        showNotification(`Your creature evolved to level ${creatureState.level}!`, 'evolution');
    }
}

function updateTimerDisplay() {
    const minutes = String(timerState.minutes).padStart(2, '0');
    const seconds = String(timerState.seconds).padStart(2, '0');
    timerDisplay.textContent = `${minutes}:${seconds}`;
}

function updateButtonStates() {
    startBtn.disabled = timerState.isRunning;
    pauseBtn.disabled = !timerState.isRunning;
    resetBtn.disabled = false;
    completeBtn.disabled = !timerState.isRunning && !timerState.isPaused;
}

function updateCreatureDisplay() {
    // Update creature emoji based on stage
    const creatureEmojis = {
        egg: '🥚',
        baby: '🐣',
        teen: '🐤',
        adult: '🐔'
    };

    const emoji = creatureEmojis[creatureState.stage] || '🥚';
    creatureDisplay.innerHTML = `<div class="text-6xl">${emoji}</div>`;

    // Update stats
    levelDisplay.textContent = creatureState.level;
    xpDisplay.textContent = creatureState.xp;
    xpNeededDisplay.textContent = creatureState.xpNeeded;

    // Update progress bar
    const progressPercent = (creatureState.xp / creatureState.xpNeeded) * 100;
    xpProgress.style.width = `${progressPercent}%`;
}

async function saveCreatureState() {
    try {
        await invoke('save_creature_state', {
            level: creatureState.level,
            xp: creatureState.xp,
            xpNeeded: creatureState.xpNeeded,
            stage: creatureState.stage
        });
    } catch (error) {
        console.error('Failed to save creature state:', error);
    }
}

async function saveTimerState() {
    try {
        await invoke('update_timer_state', {
            minutes: timerState.minutes,
            seconds: timerState.seconds,
            isRunning: timerState.isRunning,
            isPaused: timerState.isPaused,
            initialTotalSeconds: timerState.initialTotalSeconds,
            lastSelectedMinutes: lastSelectedDuration.minutes,
            lastSelectedSeconds: lastSelectedDuration.seconds
        });
    } catch (error) {
        console.error('Failed to save timer state:', error);
    }
}

async function loadFullGameState() {
    try {
        const gameState = await invoke('get_full_game_state');
        if (gameState) {
            // Load creature state
            creatureState.level = gameState.creature.level;
            creatureState.xp = gameState.creature.xp;
            creatureState.xpNeeded = gameState.creature.xp_needed;
            creatureState.stage = gameState.creature.stage;

            // Load timer state (only if not currently running)
            if (!timerState.isRunning) {
                timerState.minutes = gameState.timer.minutes;
                timerState.seconds = gameState.timer.seconds;
                timerState.initialTotalSeconds = gameState.timer.initial_total_seconds;
                lastSelectedDuration.minutes = gameState.timer.last_selected_minutes;
                lastSelectedDuration.seconds = gameState.timer.last_selected_seconds;

                updateTimerDisplay();
                updatePresetButtonStates(lastSelectedDuration.minutes, lastSelectedDuration.seconds);
            }

            // Update displays
            updateCreatureDisplay();
            updateProgressDisplay(gameState.progress);

            // Log progress for debugging
            console.log('Loaded game state:', {
                creature: gameState.creature,
                progress: gameState.progress
            });

            // Show welcome back message if user has progress
            if (gameState.progress.total_pomodoros_completed > 0) {
                const streak = gameState.progress.current_streak;
                const streakText = streak > 1 ? ` You're on a ${streak} day streak! 🔥` : '';
                showNotification(`Welcome back!${streakText}`, 'success');
            }
        }
    } catch (error) {
        console.error('Failed to load game state:', error);
        // If loading fails, use defaults (already set)
        updateProgressDisplay({
            total_pomodoros_completed: 0,
            total_xp_earned: 0,
            current_streak: 0,
            total_time_studied_seconds: 0
        });
    }
}

function updateProgressDisplay(progress) {
    if (!progress) return;

    totalPomodorosDisplay.textContent = progress.total_pomodoros_completed || 0;
    totalXpDisplay.textContent = progress.total_xp_earned || 0;
    currentStreakDisplay.textContent = progress.current_streak || 0;

    // Convert total study time from seconds to hours and minutes
    const totalHours = Math.floor((progress.total_time_studied_seconds || 0) / 3600);
    const totalMinutes = Math.floor(((progress.total_time_studied_seconds || 0) % 3600) / 60);

    if (totalHours > 0) {
        studyTimeDisplay.textContent = `${totalHours}h ${totalMinutes}m`;
    } else {
        studyTimeDisplay.textContent = `${totalMinutes}m`;
    }
}

async function getGameProgress() {
    try {
        return await invoke('get_game_progress');
    } catch (error) {
        console.error('Failed to get game progress:', error);
        return null;
    }
}

async function resetAllData() {
    try {
        // Create backup of current state before reset
        dataBackup = {
            creature: { ...creatureState },
            timer: { ...timerState },
            lastDuration: { ...lastSelectedDuration },
            progress: await getGameProgress()
        };

        // Reset data immediately
        await invoke('reset_game_data');

        // Reset local state
        creatureState.level = 1;
        creatureState.xp = 0;
        creatureState.xpNeeded = 100;
        creatureState.stage = 'egg';

        timerState.minutes = 25;
        timerState.seconds = 0;
        timerState.isRunning = false;
        timerState.isPaused = false;
        timerState.initialTotalSeconds = 1500;

        lastSelectedDuration.minutes = 25;
        lastSelectedDuration.seconds = 0;

        // Update all displays
        updateTimerDisplay();
        updateCreatureDisplay();
        updateButtonStates();
        updatePresetButtonStates(25, 0);
        updateProgressDisplay({
            total_pomodoros_completed: 0,
            total_xp_earned: 0,
            current_streak: 0,
            total_time_studied_seconds: 0
        });

        showNotificationWithAction('All data has been reset!', 'warning', 'Undo', undoDataReset);
    } catch (error) {
        console.error('Failed to reset data:', error);
        showNotification('Failed to reset data', 'error');
    }
}

// Undo function for data reset
async function undoDataReset() {
    if (!dataBackup) {
        showNotification('No backup available to restore', 'error');
        return;
    }

    try {
        // Restore the backup data to backend
        await invoke('save_full_game_state', {
            gameState: {
                creature: {
                    level: dataBackup.creature.level,
                    xp: dataBackup.creature.xp,
                    xp_needed: dataBackup.creature.xpNeeded,
                    stage: dataBackup.creature.stage
                },
                timer: {
                    minutes: dataBackup.timer.minutes,
                    seconds: dataBackup.timer.seconds,
                    is_running: dataBackup.timer.isRunning,
                    is_paused: dataBackup.timer.isPaused,
                    initial_total_seconds: dataBackup.timer.initialTotalSeconds,
                    last_selected_minutes: dataBackup.lastDuration.minutes,
                    last_selected_seconds: dataBackup.lastDuration.seconds
                },
                progress: dataBackup.progress,
                version: "1.0.0"
            }
        });

        // Restore local state
        Object.assign(creatureState, dataBackup.creature);
        Object.assign(timerState, dataBackup.timer);
        Object.assign(lastSelectedDuration, dataBackup.lastDuration);

        // Update all displays
        updateTimerDisplay();
        updateCreatureDisplay();
        updateButtonStates();
        updatePresetButtonStates(lastSelectedDuration.minutes, lastSelectedDuration.seconds);
        updateProgressDisplay(dataBackup.progress);

        // Clear backup
        dataBackup = null;

        showNotification('Data has been restored!', 'success');
    } catch (error) {
        console.error('Failed to restore data:', error);
        showNotification('Failed to restore data', 'error');
    }
}

// Make resetAllData available globally for HTML onclick
window.resetAllData = resetAllData;
