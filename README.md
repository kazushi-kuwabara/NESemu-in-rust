<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>NES Emulator in Rust</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            line-height: 1.6;
            margin: 0;
            padding: 0;
            background-color: #f4f4f9;
            color: #333;
        }
        header {
            background-color: #007acc;
            color: white;
            padding: 1em 0;
            text-align: center;
        }
        main {
            margin: 2em;
            padding: 1em;
            background: white;
            border-radius: 5px;
            box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
        }
        h1, h2 {
            color: #007acc;
        }
        footer {
            text-align: center;
            margin-top: 2em;
            font-size: 0.9em;
            color: #555;
        }
    </style>
</head>
<body>

<header>
    <h1>NES Emulator in Rust</h1>
</header>

<main>
    <section>
        <h2>Project Overview</h2>
        <p>This project is an implementation of a Nintendo Entertainment System (NES) emulator written in Rust. The emulator aims to replicate the functionality of the NES hardware to run NES games.</p>
    </section>

    <section>
        <h2>Current Status</h2>
        <ul>
            <li>Currently focused on implementing the <strong>CPU (6502)</strong>.</li>
            <li>The primary goal is to successfully run a simple game, such as <strong>Snake</strong>.</li>
            <li>The Picture Processing Unit (PPU) and other components are planned for future development.</li>
        </ul>
    </section>

    <section>
        <h2>Future Goals</h2>
        <ul>
            <li>Complete the implementation of the PPU for graphical output.</li>
            <li>Implement the APU (Audio Processing Unit) for sound emulation.</li>
            <li>Support for NES cartridges and mappers.</li>
            <li>Develop a user-friendly interface for loading and playing games.</li>
        </ul>
    </section>

    <section>
        <h2>How to Contribute</h2>
        <p>Contributions are welcome! Feel free to open issues or submit pull requests to improve the emulator or add new features.</p>
    </section>

    <section>
        <h2>License</h2>
        <p>This project is licensed under the <strong>MIT License</strong>. See the LICENSE file for more details.</p>
    </section>
</main>

<footer>
    <p>&copy; 2024 NES Emulator in Rust Project</p>
</footer>

</body>
</html>
