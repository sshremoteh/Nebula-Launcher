<div align="center">

<img src="icon.jpg" alt="Nebula Launcher Logo" width="130" style="border-radius: 28px; box-shadow: 0 12px 32px rgba(129, 140, 248, 0.4);" />

# ✨ Nebula Launcher ✨
### Minecraft: Java Edition 26.3 для Android на чистом Rust

[![Release](https://img.shields.io/badge/Release-v1.0-brightgreen.svg?style=for-the-badge&logo=android)](https://github.com/sshremoteh/Nebula-Launcher/releases/tag/v1.0)
[![Android](https://img.shields.io/badge/Android-9.0--15+-blue.svg?style=for-the-badge&logo=android)](https://github.com/sshremoteh/Nebula-Launcher)
[![Arch](https://img.shields.io/badge/Architecture-ARM64--v8a-orange.svg?style=for-the-badge&logo=arm)](https://github.com/sshremoteh/Nebula-Launcher)
[![Engine](https://img.shields.io/badge/Rust-wgpu%20%7C%20Vulkan-red.svg?style=for-the-badge&logo=rust)](https://github.com/sshremoteh/Nebula-Launcher)
[![Website](https://img.shields.io/badge/Website-GitHub%20Pages-blueviolet.svg?style=for-the-badge&logo=github)](https://sshremoteh.github.io/Nebula-Launcher/)

<p align="center">
  <b>Высокопроизводительный нативный клиент Minecraft: Java Edition для Android смартфонов и планшетов.</b><br>
  Непревзойденный FPS на Vulkan, сенсорное управление Bedrock Touch, поддержка Java серверов и моддинга.
</p>

[📥 **СКАЧАТЬ APK (v1.0 Release)**](https://github.com/sshremoteh/Nebula-Launcher/releases/latest/download/Nebula-Launcher-v1.0.apk) • [🌐 **Официальный сайт**](https://sshremoteh.github.io/Nebula-Launcher/) • [📝 **Список изменений**](https://github.com/sshremoteh/Nebula-Launcher/releases)

---

</div>

## 🌟 Ключевые особенности

<table>
  <tr>
    <td width="50%">
      <h3>🚀 Невероятная производительность</h3>
      <ul>
        <li>Движок полностью переписан на <b>Rust</b> с использованием <code>wgpu</code>.</li>
        <li>Аппаратное ускорение через <b>Vulkan</b> и <b>OpenGL ES 3.0</b>.</li>
        <li>Стабильные <b>60–120 FPS</b> и потребление ОЗУ от 2.5 ГБ.</li>
      </ul>
    </td>
    <td width="50%">
      <h3>🎮 Сенсорное управление Bedrock Touch</h3>
      <ul>
        <li>4 схемы управления: аналоговый джойстик или классический D-Pad.</li>
        <li>Прицеливание тапом по экрану или перекрестием.</li>
        <li>Свободное перемещение, изменение размера и прозрачности кнопок.</li>
      </ul>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <h3>🌐 Сетевая игра Java Edition</h3>
      <ul>
        <li>Прямое подключение к Java-серверам (Hypixel, локальные сервера LAN).</li>
        <li>Поддержка одиночной игры без подключения к интернету.</li>
        <li>Удобное сохранение и экспорт миров через системный проводник.</li>
      </ul>
    </td>
    <td width="50%">
      <h3>🌌 Современный лаунчер Nebula 1.0</h3>
      <ul>
        <li>Глубокий космический градиентный дизайн и неоновое свечение.</li>
        <li>Аппаратные векторные GPU-иконки без артефактов и битых символов.</li>
        <li>Детальный прогресс загрузки: точные проценты, мегабайты и файлы.</li>
      </ul>
    </td>
  </tr>
</table>

---

## 📸 Скриншоты геймплея на телефоне

| Джойстик и перекрестие | D-Pad и сенсорное прицеливание |
| :---: | :---: |
| ![Fox in a forest](docs/media/android/joystick-fox.png) | ![River and forest](docs/media/android/dpad-river.png) |

| Заснеженные биомы | Hypixel Arcade (Java Multiplayer) |
| :---: | :---: |
| ![Snow landscape](docs/media/android/snow-landscape.png) | ![Hypixel Arcade](docs/media/android/hypixel-arcade.png) |

| Вид от 3-го лица | Пустынный ландшафт |
| :---: | :---: |
| ![Third-person desert](docs/media/android/third-person-desert.png) | ![Desert panorama](docs/media/android/desert-panorama.png) |

---

## 📲 Установка на Android

1. Скачайте актуальный установочный файл [**Nebula-Launcher-v1.0.apk**](https://github.com/sshremoteh/Nebula-Launcher/releases/latest/download/Nebula-Launcher-v1.0.apk).
2. Запустите установку APK на вашем Android-устройстве. При необходимости разрешите «Установку из неизвестных источников».
3. Откройте **Nebula Launcher**.
4. Введите свой никнейм в левой панели (сохраняется автоматически).
5. Нажмите кнопку **«СКАЧАТЬ MINECRAFT 26.3»** для первой загрузки игровых ресурсов (прогресс загрузки отображается в реальном времени с точными процентами и МБ).
6. После завершения загрузки нажмите большую зеленую кнопку **«И Г Р А Т Ь»**!

---

## ⚙️ Системные требования

| Параметр | Минимальные требования | Рекомендуемые |
| :--- | :--- | :--- |
| **ОС** | Android 9.0 (Pie, API 28) | Android 12–15+ (API 31–35) |
| **Архитектура** | 64-bit ARM (`arm64-v8a`) | 64-bit ARM (`arm64-v8a`) |
| **Оперативная память** | 3 ГБ RAM | 6–8 ГБ RAM и выше |
| **Графический API** | OpenGL ES 3.0 | Vulkan 1.1+ |
| **Память устройства** | ~500 МБ свободного места | 2 ГБ (для миров и паков) |

---

## 🛠 Самостоятельная сборка проекта (Linux / Termux)

В репозиторий включен автономный скрипт [`build.sh`](build.sh):

```bash
# Клонирование репозитория
git clone https://github.com/sshremoteh/Nebula-Launcher.git
cd Nebula-Launcher

# Быстрая проверка компиляции кода
./build.sh check

# Сборка и упаковка оптимизированного APK (arm64-v8a)
./build.sh apk fast

# Готовый подписанный файл появится в:
# .local/android/apk/nebula-launcher.apk
```

---

## 📄 Лицензия и благодарности

* Проект распространяется по лицензии открытого исходного кода.
* Движок основан на исследовательской базе Horizon Rust Client для Minecraft Java Edition 26.3.
* Все права на Minecraft принадлежат Mojang AB / Microsoft. Данный проект не связан с Mojang AB.

<div align="center">
  <sub>Создано с ❤️ для сообщества любителей мобильного Minecraft.</sub>
</div>
