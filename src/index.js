// import { invoke } from '@tauri-apps/api/tauri';
// import { appWindow } from '@tauri-apps/api/window';
// import { emit, listen, once } from '@tauri-apps/api/event';
const { invoke } = window.__TAURI__.tauri;
const { appWindow } = window.__TAURI__.window;
const { emit, listen, once } = window.__TAURI__.event;

/**
 * 设置
 */
let config = {
    preset: undefined,
    enable: true,
    key_down_transition_duration: 0,
    key_up_transition_duration: 100,
    mouse_move_enable: false,
    mouse_move_radius_px: 50,
    mouse_move_transition_duration: 100
};
/**
 * 服务器端口号
 */
let port;
/**
 * 版本号
 */
let version;

// 元素
/**
 * 标题版本部分
 */
let versionText = document.querySelector("#version");
/**
 * 总开关
 */
let mainSwitch = document.querySelector("#main-switch .switch");
/**
 * 鼠标移动开关
 */
let mouseSwitch = document.querySelector("#mouse-move-switch .switch")

/**
 * 键盘按下滚动条
 */
let keyDownRange = document.querySelector("#key_down_transition");
/**
 * 键盘抬起滚动条
 */
let keyUpRange = document.querySelector("#key_up_transition");
/**
 * 鼠标区间滚动条
 */
let mouseRadiusRange = document.querySelector("#mouse_move_radius");
/**
 * 鼠标移动滚动条
 */
let mouseMoveRange = document.querySelector("#mouse_move_transition");
/**
 * 键盘具体设置组
 */
let keysSettingSet = document.querySelector(".setting-set.keys");
/**
 * 鼠标开关设置组
 */
let mouseSwitchSettingSet = document.querySelector(".setting-set.mouse-switch");
/**
 * 鼠标具体设置组
 */
let mousesSettingSet = document.querySelector(".setting-set.mouses");
/**
 * 右上角叉叉
 */
let close = document.querySelector("#close");
/**
 * 预设select
 */
let presetSelect = document.querySelector("#preset_select");
/**
 * 服务器状态行
 */
let server = document.querySelector("#server");
/**
 * 复制端口的元素
 */
let portCopyEle = document.querySelector("#port");
/**
 * b站主页
 */
let creditBilibili = document.querySelector("#bilibili");
/**
 * github主页
 */
let creditGithub = document.querySelector("#github");


// 滚动条实例
/**
 * @description: 键盘按下滚动条实例
 * @type {Slider | undefined}
 */
let keyDownSlider;
/**
 * @description: 键盘抬起滚动条实例
 * @type {Slider | undefined}
 */
let keyUpSlider;
/**
 * @description: 鼠标区间滚动条实例
 * @type {Slider | undefined}
 */
let mouseRadiusSlider;
/**
 * @description: 鼠标移动滚动条实例
 * @type {Slider | undefined}
 */
let mouseMoveSlider;


/**
 * @description: 存储滚动条旧值和实例的Object
 */
let rangeObject;

/**
 * 预设列表
 */
let presets;
/**
 * 上一个选择的预设
 */
let lastPreset;

// DOM加载完成后
document.addEventListener("DOMContentLoaded", async () => {
    // 实例化滚动条
    keyDownSlider = new Slider(keyDownRange);
    keyUpSlider = new Slider(keyUpRange);
    mouseRadiusSlider = new Slider(mouseRadiusRange);
    mouseMoveSlider = new Slider(mouseMoveRange);
    rangeObject = {
        key_down_transition_duration: {
            old: 0,
            instance: keyDownSlider
        },
        key_up_transition_duration: {
            old: 0,
            instance: keyUpSlider
        },
        mouse_move_radius_px: {
            old: 0,
            instance: mouseRadiusSlider
        },
        mouse_move_transition_duration: {
            old: 0,
            instance: mouseMoveSlider
        }
    }

    // 获取并加载预设
    presets = await getPresets();
    initPresets();

    // 获取设置、服务器端口号和版本号，初始化设置
    config = await getConfig();
    port = await getPort();
    version = await getVersion();
    initConfigs();

    // 初始化输入事件
    initInputEvents();
    // 取消所有禁用
    setAllEnable();

    appWindow.show();
});

/**
 * @description: 初始化预设列表
 */
function initPresets() {
    presetSelect.options.length = 1;
    for (let preset of presets) {
        let option = document.createElement("option");
        option.innerText = option.value = preset;
        presetSelect.appendChild(option);
    }
}

/**
 * @description: 初始化输入事件
 */
function initInputEvents() {
    close.addEventListener('click', () => closeWindow());
    presetSelect.addEventListener('change', () => presetChanged());
    mainSwitch.addEventListener('click', () => toggleEnable());
    mouseSwitch.addEventListener('click', () => toggleMouseEnable());
    keyDownRange.addEventListener(
        'change',
        (event) => onSliderChange(
            'key_down_transition_duration',
            event.target.valueAsNumber));
    keyUpRange.addEventListener(
        'change',
        (event) => onSliderChange(
            'key_up_transition_duration',
            event.target.valueAsNumber));
    mouseRadiusRange.addEventListener(
        'change',
        (event) => onSliderChange(
            'mouse_move_radius_px',
            event.target.valueAsNumber));
    mouseMoveRange.addEventListener(
        'change',
        (event) => onSliderChange(
            'mouse_move_transition_duration',
            event.target.valueAsNumber));
    // 点击端口后复制到剪切板
    portCopyEle.addEventListener("click", async () => {
        if (!port) return;
        await navigator.clipboard.writeText(port);
        let copyIcon = server.querySelector(".copy-icon");
        copyIcon.classList.remove('hide');
        await sleep(1500);
        copyIcon.classList.add('hide');
    });
    // 打开B站空间
    creditBilibili.addEventListener("click", () => openCredit("bilibili"));
    // 打开github主页
    creditGithub.addEventListener("click", () => openCredit("github"));
}

/**
 * @description: 初始化设置
 */
function initConfigs() {
    if (config.preset) {
        for (let i = 0; i < presetSelect.options.length; i++) {
            if (presetSelect.options[i].value == config.preset) {
                presetSelect.selectedIndex = i;
                lastPreset = presetSelect.options[i].value;
            }
        }
    }

    if (config.enable) {
        mainSwitch.classList.add('active');
    }
    if (config.mouse_move_enable) {
        mouseSwitch.classList.add('active');
    }
    mainSwitchAnimation();

    keyDownSlider.setValue(config.key_down_transition_duration);
    keyUpSlider.setValue(config.key_up_transition_duration);
    mouseRadiusSlider.setValue(config.mouse_move_radius_px);
    mouseMoveSlider.setValue(config.mouse_move_transition_duration);
    rangeObject.key_down_transition_duration.old = config.key_down_transition_duration;
    rangeObject.key_up_transition_duration.old = config.key_up_transition_duration;
    rangeObject.mouse_move_radius_px.old = config.mouse_move_radius_px;
    rangeObject.mouse_move_transition_duration.old = config.mouse_move_transition_duration;

    versionText.innerText = `v${version}`;

    if (port) {
        server.querySelector(".dot").classList.add("on");
        portCopyEle.classList.remove("disabled");
        portCopyEle.innerText = port;
    }
}

/**
 * @description: 更改预设后
 */
async function presetChanged() {
    let saved = await updateConfig('preset', presetSelect.options[presetSelect.selectedIndex].value);
    if (saved) {
        lastPreset = presetSelect.options[presetSelect.selectedIndex].value;
    } else {
        for (let i = 0; i < presetSelect.options.length; i++) {
            if (presetSelect.options[i].value == lastPreset) {
                presetSelect.selectedIndex = i;
                break;
            }
        }
    }
}

/**
 * 是否能按下总开关
 */
let canToggleEnable = true; // TODO: 默认关闭，加载完成后打开
/**
 * @description: 按下总开关后
 */
async function toggleEnable() {
    if (!canToggleEnable) return;
    let saved = await updateConfig('enable', !config.enable);
    // console.log(savedConfig);
    // console.log(newConfig);
    if (saved) {
        mainSwitchAnimation();
    }
}

/**
 * @description: 主开关动画
 */
async function mainSwitchAnimation() {
    if (config.enable) {
        mainSwitch.classList.add('active');
        keysSettingSet.classList.remove('off');
        await sleep(150);
        mouseSwitchSettingSet.classList.remove('off');
        await sleep(150);
        if (config.mouse_move_enable) {
            mousesSettingSet.classList.remove('off');
        }
    } else {
        mainSwitch.classList.remove('active');
        keysSettingSet.classList.add('off');
        await sleep(150);
        mouseSwitchSettingSet.classList.add('off');
        await sleep(150);
        mousesSettingSet.classList.add('off');
    }
}

/**
 * 是否能按下鼠标启动开关
 */
let canToggleMouseEnable = true; // TODO: 默认关闭，加载完成后打开
/**
 * @description: 按下鼠标功能开关后
 */
async function toggleMouseEnable() {
    if (!canToggleMouseEnable) return;
    let saved = await updateConfig('mouse_move_enable', !config.mouse_move_enable);
    // console.log(savedConfig);
    // console.log(newConfig);
    if (saved) {
        mouseSwitchAnimation();
    }
}

/**
 * @description: 鼠标开关动画
 * @return {Promise<void>}
 */
async function mouseSwitchAnimation() {
    if (config.mouse_move_enable) {
        mouseSwitch.classList.add('active');
        mousesSettingSet.classList.remove('off');
    } else {
        mouseSwitch.classList.remove('active');
        mousesSettingSet.classList.add('off');
    }
}

/**
 * @description: 滚动条类型的输入的值变化后，根据key修改value
 * @param {'key_down_transition_duration' | 'key_up_transition_duration' | 'mouse_move_radius_px' | 'mouse_move_transition_duration'} key 滚动条所指向的配置项
 * @param {number} value 滚动条的值
 */
async function onSliderChange(key, value) {
    let saved = await updateConfig(key, value);
    if (saved) {
        rangeObject[key].old = value;
    } else {
        rangeObject[key].instance.setValue(rangeObject[key].old);
    }
}

/**
 * @description: 更新设置
 * @param {'preset' | 'enable' | 'mouse_move_enable' | 'key_down_transition_duration' | 'key_up_transition_duration' | 'mouse_move_radius_px' | 'mouse_move_transition_duration'} key 设置的键
 * @param {boolean | number | undefined} value 设置的值
 * @param {Function} disable 禁用元素的方法
 * @param {Function} enable 启用元素的方法
 * @return {Promise<boolean>}
 */
async function updateConfig(key, value, disable = setAllDisable, enable = setAllEnable) {
    let keyFound = false;
    for (let configKey of Object.keys(config)) {
        if (key == configKey) {
            keyFound = true;
            break;
        }
    }
    if (!keyFound) {
        return false;
    }
    // 保存前将所有输入禁用
    disable();
    // 保存设置
    let newConfig = {
        ...config,
        [key]: value,
    }
    let savedConfig = await setConfig(newConfig);
    if (savedConfig) {
        config = newConfig;
    }
    // 保存后200毫秒后取消禁用
    await sleep(200);
    enable();
    return savedConfig;
}

/**
 * @description: 设置所有输入的禁用状态（在更改设置时）
 */
function setAllDisable() {
    presetSelect.setAttribute('disabled', true);
    canToggleEnable = false;
    mainSwitch.classList.add('disabled');
    canToggleMouseEnable = false;
    mouseSwitch.classList.add('disabled');
    keyDownRange.querySelector('.fir-range').setAttribute('disabled', true);
    keyDownRange.classList.add('disabled');
    keyUpRange.querySelector('.fir-range').setAttribute('disabled', true);
    keyUpRange.classList.add('disabled');
    mouseRadiusRange.querySelector('.fir-range').setAttribute('disabled', true);
    mouseRadiusRange.classList.add('disabled');
    mouseMoveRange.querySelector('.fir-range').setAttribute('disabled', true);
    mouseMoveRange.classList.add('disabled');
}


/**
 * @description: 取消所有输入的禁用状态（更改设置后）
 */
function setAllEnable() {
    presetSelect.removeAttribute('disabled');
    canToggleEnable = true;
    mainSwitch.classList.remove('disabled');
    canToggleMouseEnable = true;
    mouseSwitch.classList.remove('disabled');
    keyDownRange.querySelector('.fir-range').removeAttribute('disabled');
    keyDownRange.classList.remove('disabled');
    keyUpRange.querySelector('.fir-range').removeAttribute('disabled');
    keyUpRange.classList.remove('disabled');
    mouseRadiusRange.querySelector('.fir-range').removeAttribute('disabled');
    mouseRadiusRange.classList.remove('disabled');
    mouseMoveRange.querySelector('.fir-range').removeAttribute('disabled');
    mouseMoveRange.classList.remove('disabled');
}


/*************************************/
/**           Tauri 交互             */
/*************************************/
/**
 * @description: 保存设置
 * @param {{preset: string | undefined, enable: boolean, key_down_transition_duration: number, key_up_transition_duration: number, mouse_move_enable: boolean, mouse_move_radius_px: number, mouse_move_transition_duration: number}} newConfig
 * @return {Promise<boolean>}
 */
async function setConfig(newConfig) {
    return await invoke("set_config", { newConfig });
}

/**
 * @description: 获取设置
 * @return {Promise<{preset: string | undefined, enable: boolean, key_down_transition_duration: number, key_up_transition_duration: number, mouse_move_enable: boolean, mouse_move_radius_px: number, mouse_move_transition_duration: number}>}
 */
async function getConfig() {
    return await invoke("get_config");
}

/**
 * @description: 获取服务器端口
 * @return {Promise<number>}
 */
async function getPort() {
    return await invoke("get_port");
}

/**
 * @description: 获取版本号
 * @return {Promise<string>}
 */
async function getVersion() {
    return await invoke("get_version");
}

/**
 * @description: 关闭窗口
 * @return {Promise<void>}
 */
async function closeWindow() {
    return await invoke("close_window");
}

/**
 * @description: 获取预设列表
 * @return {Promise<Array<string>>}
 */
async function getPresets() {
    return await invoke("get_presets");
}

/**
 * @description: 打开个人页面
 * @param {string} site
 */
async function openCredit(site) {
    await invoke("open_credit", { site });
}

/***********************/
/*        其他         */
/***********************/
/**
 * @description: 阻塞一定时长（毫秒）
 * @param {number} time 毫秒数
 * @return {Promise<void>}
 */
function sleep(time) {
    return new Promise((res, _) => {
        setTimeout(() => {
            res();
        }, time);
    });
}