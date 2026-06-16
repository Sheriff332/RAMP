### 🏗️ The 4-Folder Rule
This project follows a strict decoupled architecture to ensure logic is pure and side effects are contained.

1. **IO (`src/io.rs`)**: The Senses. How the world talks to us.
2. **Logic (`src/logic.rs`)**: The Brain. Pure calculations.
3. **Storage (`src/storage.rs`)**: The Vault. Data structures and persistence.
4. **Main (`src/main.rs`)**: The Bouncer. Orchestrates the flow.

### 🚀 Quick Start
1. Map the flow in a nested flowchart.
2. Define the 'Vault' structs.
3. Write the 'Brain' transforms.
4. Plug it all into the 'Bouncer'.

# ⚖️ Main 
**Rule:** Connect, don't Compute.

- **Job 1:** Authenticate and validate data from **IO**.
- **Job 2:** Pass valid data to **Logic**.
- **Job 3:** Take results from Logic and send to **Storage**.
