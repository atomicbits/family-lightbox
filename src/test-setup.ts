// The tests of texts were written in Dutch: they run in Dutch, a test of English sets it itself (F011/S01).
import { beforeEach } from "vitest";
import { setLanguage } from "./i18n";

beforeEach(() => setLanguage("nl"));
