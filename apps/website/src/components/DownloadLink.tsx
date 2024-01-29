import { createSignal } from "solid-js/types/server/reactive.js";
import Apple from "../assets/Apple";
import Linux from "../assets/Linux";
import Windows from "../assets/Windows";
import yaml from "js-yaml";
import { createResource } from "solid-js/types/server/rendering.js";

const getOs = () => {
  if (window.navigator.userAgent.toLowerCase().includes("windows")) {
    return "Windows";
  } else if (window.navigator.userAgent.toLowerCase().includes("darwin")) {
    return "MacOS";
  } else if (window.navigator.userAgent.toLowerCase().includes("linux")) {
    return "Linux";
  } else {
    return "Unknown";
  }
};

async function getDownloadLink() {
  const getPath = () => {
    if (getOs() === "Windows") {
      return "https://cdn-raw.gdl.gg/launcher/alpha.yml";
    } else if (getOs() === "MacOS") {
      return "https://cdn-raw.gdl.gg/launcher/alpha-mac.yml";
    } else {
      return "https://cdn-raw.gdl.gg/launcher/alpha-linux.yml";
    }
  };
  const response = await fetch(getPath());
  const data = await response.text();
  const doc = yaml.load(data) as {
    version: string;
    files: Array<{
      url: string;
      sha512: string;
      size: number;
    }>;
    path: string;
    sha512: string;
    releaseDate: string;
  };
  const downloadLink = `https://cdn-raw.gdl.gg/launcher/${doc.path}`;
  return downloadLink;
}

export const DownloadLink = () => {
  const [data] = createResource(getDownloadLink);
  return (
    <a href={data()} class="flex items-center gap-2">
      <span>DOWNLOAD FOR</span>
      {getOs() === "Windows" ? (
        <Windows />
      ) : getOs() === "MacOS" ? (
        <Apple />
      ) : (
        <Linux />
      )}
    </a>
  );
};