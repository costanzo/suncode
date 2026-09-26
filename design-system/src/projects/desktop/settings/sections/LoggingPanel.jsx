import { useRef, useState } from "react";

import { Button } from "../../../../components/universal/button/index.js";
import { SingleDropdown } from "../../../../components/universal/dropdown/index.js";

import { Icon } from "../../../../shared/Icon.jsx";
import { SettingRow } from "../components/SettingRow.jsx";

export function LoggingPanel({ onSave }) {
  const [directory, setDirectory] = useState("~/.suncode/logs");
  const [imageDirectory, setImageDirectory] = useState("~/.suncode/images");
  const directoryInputRef = useRef(null);
  const imageDirectoryInputRef = useRef(null);
  const chooseDirectory = async (setValue, fallbackInputRef) => {
    if (!("showDirectoryPicker" in window)) {
      fallbackInputRef.current?.click();
      return;
    }
    try {
      const handle = await window.showDirectoryPicker({ mode: "readwrite" });
      setValue(`~/${handle.name}`);
    } catch (error) {
      if (error?.name !== "AbortError") fallbackInputRef.current?.click();
    }
  };
  const chooseFallbackDirectory = (event, setValue) => {
    const relativePath = event.target.files?.[0]?.webkitRelativePath;
    const folderName = relativePath?.split("/")[0];
    if (folderName) setValue(`~/${folderName}`);
    event.target.value = "";
  };
  return (
    <div className="settings-panel-content">
      <div className="settings-panel-heading">
        <h2>Logging</h2>
        <p>Control diagnostic detail and how long local log files are kept.</p>
      </div>
      <div className="settings-panel-section">
        <span className="settings-section-label">Diagnostic output</span>
        <SettingRow label="Minimum level" hint="Lower levels include more diagnostic detail.">
          <SingleDropdown
            options={["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "OFF"]}
            initialValue="INFO"
            ariaLabel="Minimum log level"
            className="settings-dropdown"
          />
        </SettingRow>
        <SettingRow
          label="Log directory"
          hint="Agent and desktop log files are written to this folder."
        >
          <div className="settings-directory-field">
            <input
              className="field mono"
              value={directory}
              onChange={(event) => setDirectory(event.target.value)}
              aria-label="Log directory"
              spellCheck="false"
            />
            <button
              type="button"
              aria-label="Choose log directory"
              title="Choose folder"
              onClick={() => chooseDirectory(setDirectory, directoryInputRef)}
            >
              <Icon name="folder" size={16} />
            </button>
            <input
              ref={directoryInputRef}
              type="file"
              webkitdirectory=""
              aria-hidden="true"
              tabIndex="-1"
              onChange={(event) => chooseFallbackDirectory(event, setDirectory)}
            />
          </div>
        </SettingRow>
        <SettingRow
          label="Maximum file size"
          hint="Rotate each log file when it reaches this size."
        >
          <label className="settings-unit-field">
            <input
              className="field mono"
              type="number"
              min="1"
              max="1000"
              step="1"
              defaultValue="10"
              aria-label="Maximum file size in megabytes"
            />
            <span>MB</span>
          </label>
        </SettingRow>
        <SettingRow
          label="Retained backups"
          hint="Number of rotated backups to keep, from 0 to 100."
        >
          <input
            className="field settings-number"
            type="number"
            min="0"
            max="100"
            defaultValue="5"
            aria-label="Retained backups"
          />
        </SettingRow>
      </div>
      <div className="settings-actions">
        <Button variant="primary" size="sm" onClick={onSave}>
          Save logging settings
        </Button>
        <span className="settings-save-status" role="status">
          Local settings
        </span>
      </div>
      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">Image storage</span>
        <SettingRow label="Image directory" hint="Directory for saving images.">
          <div className="settings-directory-field">
            <input
              className="field mono"
              value={imageDirectory}
              onChange={(event) => setImageDirectory(event.target.value)}
              aria-label="Image directory"
              spellCheck="false"
            />
            <button
              type="button"
              aria-label="Choose image directory"
              title="Choose folder"
              onClick={() => chooseDirectory(setImageDirectory, imageDirectoryInputRef)}
            >
              <Icon name="folder" size={16} />
            </button>
            <input
              ref={imageDirectoryInputRef}
              type="file"
              webkitdirectory=""
              aria-hidden="true"
              tabIndex="-1"
              onChange={(event) => chooseFallbackDirectory(event, setImageDirectory)}
            />
          </div>
        </SettingRow>
      </div>
      <div className="settings-actions">
        <Button variant="primary" size="sm" onClick={onSave}>
          Save image location
        </Button>
        <span className="settings-save-status" role="status">
          Local settings
        </span>
      </div>
    </div>
  );
}
