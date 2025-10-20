#!/usr/bin/env node
':'; // ; cat "$0" | node --input-type=module - $@ ; exit $?
// -*- mode: js; indent-tabs-mode: nil; tab-width: 4 -*-

import { readFileSync, writeFileSync } from 'fs';
import { basename } from 'path';

const TARGET_PROPERTY_ORDER = [
  // Core Architecture
  'arch',
  'cpu',
  'features',
  'need-explicit-cpu',
  // LLVM Backend
  'llvm-target',
  'llvm-abiname',
  'llvm-floatabi',
  'llvm-mcount-intrinsic',
  'llvm-args',
  'data-layout',
  // Target Environment
  'target-endian',
  'target-pointer-width',
  'target-c-int-width',
  'os',
  'env',
  'vendor',
  'abi',
  'target-family',
  // Memory & Threading
  'max-atomic-width',
  'min-global-align',
  'atomic-cas',
  'singlethread',
  'has-thread-local',
  'dll-tls-export',
  'disable-redzone',
  'frame-pointer',
  // Compilation Strategy
  'panic-strategy',
  'relocation-model',
  'code-model',
  'tls-model',
  'merge-functions',
  'trap-unreachable',
  'no-builtins',
  'relax-elf-relocations',
  'plt-by-default',
  // Linker & Executables
  'linker',
  'linker-flavor',
  'lld-flavor',
  'executables',
  'exe-suffix',
  'dll-prefix',
  'dll-suffix',
  'staticlib-prefix',
  'staticlib-suffix',
  'dynamic-linking',
  'only-cdylib',
  'position-independent-executables',
  'static-position-independent-executables',
  'relro-level',
  'has-rpath',
  'no-default-libraries',
  // Link Objects & Arguments
  'pre-link-objects',
  'pre-link-objects-fallback',
  'pre-link-objects-self-contained',
  'post-link-objects',
  'post-link-objects-fallback',
  'post-link-objects-self-contained',
  'link-self-contained',
  'pre-link-args',
  'pre-link-args-json',
  'late-link-args',
  'late-link-args-dynamic',
  'late-link-args-static',
  'late-link-args-dynamic-json',
  'late-link-args-static-json',
  'post-link-args',
  'post-link-args-json',
  'link-env',
  'link-env-remove',
  'link-script',
  'archive-format',
  'limit-rdylib-exports',
  'override-export-symbols',
  // C Runtime
  'crt-static-allows-dylibs',
  'crt-static-default',
  'crt-static-respected',
  'crt-objects-fallback',
  // Function Calls & Stack
  'stack-probes',
  'function-sections',
  'mcount',
  'abi-return-struct-as-int',
  'c-enum-min-bits',
  'entry-name',
  'entry-abi',
  // Debug & Development
  'debuginfo-kind',
  'default-dwarf-version',
  'emit-debug-gdb-scripts',
  'supported-split-debuginfo',
  'split-debuginfo',
  'supported-sanitizers',
  'default-visibility',
  'eh-frame-header',
  'requires-lto',
  'allows-weak-linkage',
  'obj-is-bitcode',
  'bitcode-llvm-cmdline',
  // Platform Detection
  'is-like-android',
  'is-like-aix',
  'is-like-darwin',
  'is-like-solaris',
  'is-like-windows',
  'is-like-msvc',
  'is-like-wasm',
  // Codegen & Assembly
  'default-codegen-units',
  'default-codegen-backend',
  'rustc-abi',
  'binary-format',
  'default-uwtable',
  'requires-uwtable',
  'asm-args',
  // Metadata
  'metadata',
];

const log = {
  info: (msg) => console.log(msg),
  warn: (msg) => console.warn(msg),
  error: (msg) => console.error(msg),
};

const readJson = (file) => JSON.parse(readFileSync(file, 'utf8'));
const writeJson = (file, data) => writeFileSync(file, JSON.stringify(data, null, 2) + '\n');

const parseArgs = (argv) => {
  const out = { schemaFile: null, files: [] };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--schema' && i + 1 < argv.length) {
      out.schemaFile = argv[++i];
      continue;
    }
    out.files.push(a);
  }
  return out;
};

const isType = (val, type) => {
  if (type === 'null') return val === null;
  if (type === 'array') return Array.isArray(val);
  if (type === 'integer') return Number.isInteger(val);
  return typeof val === type;
};

const matchesSchemaType = (value, sch) => {
  if (!sch) return true;
  if (sch.$ref) return true; // ref resolution not implemented here
  if (Array.isArray(sch.type)) return sch.type.some((t) => isType(value, t));
  if (sch.type) return isType(value, sch.type);
  return true;
};

/**
 * Validates a value against a JSON Schema (common subset):
 * - type (including unions), anyOf
 * - object: properties, required, additionalProperties
 * - array: items (single schema or tuple)
 * Returns a flat array of error strings; empty array means valid.
 */
const validateValue = (value, schema, path = '') => {
  const errors = [];
  if (!schema) return errors;

  // anyOf
  if (schema.anyOf) {
    const ok = schema.anyOf.some((sub) => matchesSchemaType(value, sub));
    if (!ok) {
      errors.push(`${path || '<root>'}: value doesn't match anyOf`);
      return errors; // can't safely recurse into a specific branch
    }
  }

  // basic type check (including unions)
  if (schema.type) {
    const ok = Array.isArray(schema.type) ? schema.type.some((t) => isType(value, t)) : isType(value, schema.type);
    if (!ok) {
      const want = Array.isArray(schema.type) ? schema.type.join(' | ') : schema.type;
      errors.push(`${path || '<root>'}: expected ${want}, got ${Array.isArray(value) ? 'array' : typeof value}`);
      return errors;
    }
  }

  // object validation
  const isObjType =
    schema.type === 'object' ||
    (Array.isArray(schema.type) && schema.type.includes('object')) ||
    schema.properties ||
    schema.required ||
    schema.additionalProperties !== undefined;

  if (isObjType && typeof value === 'object' && value !== null && !Array.isArray(value)) {
    const props = schema.properties || {};
    const required = schema.required || [];
    const additional = schema.additionalProperties; // true | false | schema | undefined

    for (const k of required) {
      if (!(k in value)) errors.push(`${path || '<root>'}.${k}: required property missing`);
    }

    for (const [k, v] of Object.entries(value)) {
      if (k in props) {
        errors.push(...validateValue(v, props[k], path ? `${path}.${k}` : k));
      } else {
        if (additional === false) {
          errors.push(`${path || '<root>'}.${k}: unknown property (additionalProperties=false)`);
        } else if (additional && typeof additional === 'object') {
          errors.push(...validateValue(v, additional, path ? `${path}.${k}` : k));
        }
      }
    }
  }

  // array validation
  const isArrType =
    schema.type === 'array' ||
    (Array.isArray(schema.type) && schema.type.includes('array')) ||
    schema.items !== undefined;

  if (isArrType && Array.isArray(value)) {
    if (Array.isArray(schema.items)) {
      // tuple form: validate up to items.length
      schema.items.forEach((itemSchema, i) => {
        if (i < value.length) {
          errors.push(...validateValue(value[i], itemSchema, `${path}[${i}]`));
        }
      });
      // NOTE: additionalItems not enforced; add if needed.
    } else if (schema.items && typeof schema.items === 'object') {
      value.forEach((item, i) => {
        errors.push(...validateValue(item, schema.items, `${path}[${i}]`));
      });
    }
  }

  return errors;
};

const orderTargetProperties = (obj, schema, filePath) => {
  if (schema) {
    const errs = validateValue(obj, schema, '');
    if (errs.length) {
      throw new Error(`Schema validation failed for ${filePath}:\n  ${errs.join('\n  ')}`);
    }
  } else {
    const unknown = Object.keys(obj).filter((k) => !TARGET_PROPERTY_ORDER.includes(k));
    if (unknown.length) {
      throw new Error(
        `Unknown target properties: ${unknown.sort().join(', ')}\n` +
          `This may indicate outdated field names or typos in the target specification.`,
      );
    }
  }

  // strict: every present key must be in the order list
  const leftovers = Object.keys(obj).filter((k) => !TARGET_PROPERTY_ORDER.includes(k));
  if (leftovers.length) {
    throw new Error(
      `Property list missing ordering for: ${leftovers.sort().join(', ')}.\n` +
        `Add these to TARGET_PROPERTY_ORDER to avoid dropping fields.`,
    );
  }

  const ordered = {};
  for (const k of TARGET_PROPERTY_ORDER) if (k in obj) ordered[k] = obj[k];
  return ordered;
};

const loadSchemaIfAny = (schemaFile) => {
  if (!schemaFile) return null;
  try {
    const s = readJson(schemaFile);
    log.info(`✅ JSON Schema validation enabled: ${basename(schemaFile)}`);
    return s;
  } catch (e) {
    log.warn(`⚠️  Failed to load schema file ${schemaFile}: ${e.message}`);
    return null;
  }
};

const formatTargetFile = (filePath, schema) => {
  const target = readJson(filePath);
  const ordered = orderTargetProperties(target, schema, filePath);
  writeJson(filePath, ordered);
  log.info(`✅ ${basename(filePath)} formatted successfully`);
};

const main = () => {
  const args = parseArgs(process.argv.slice(2));
  const schema = loadSchemaIfAny(args.schemaFile);

  if (args.files.length === 0) {
    log.error('Rust Target Specification JSON Formatter');
    log.error('Usage: ./format-spec-target.mjs [--schema schema.json] file1.json file2.json ...');
    log.error('Example: find . -name "risc*.json" -exec ./format-spec-target.mjs --schema target-schema.json {} +');
    process.exit(1);
  }

  for (const file of args.files) {
    try {
      formatTargetFile(file, schema);
    } catch (e) {
      log.error(`❌ Error formatting ${file}: ${e.message}`);
    }
  }
};

main();
