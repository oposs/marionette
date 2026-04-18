import Ajv from 'ajv';
import addFormats from 'ajv-formats';
import * as yaml from 'js-yaml';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';

/**
 * Load all OpenAPI YAML schema files, merge definitions into a single
 * $defs map, rewrite cross-file $ref values, and create an AJV instance
 * for validating protocol messages.
 */

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const SCHEMA_DIR = path.resolve(__dirname, '../../../spec/schemas');
const SCHEMA_FILES = ['message.yaml', 'component.yaml', 'data.yaml', 'common.yaml'];

interface SchemaObject {
	[key: string]: unknown;
}

/**
 * Recursively rewrite cross-file $ref values like "data.yaml#/PatchOperation"
 * to internal format "#/$defs/PatchOperation".
 */
function rewriteRefs(obj: unknown): unknown {
	if (obj === null || typeof obj !== 'object') return obj;

	if (Array.isArray(obj)) {
		return obj.map(rewriteRefs);
	}

	const result: SchemaObject = {};
	for (const [key, value] of Object.entries(obj as SchemaObject)) {
		if (key === '$ref' && typeof value === 'string') {
			// Rewrite cross-file refs: "filename.yaml#/TypeName" -> "#/$defs/TypeName"
			// Also rewrite within-file refs: "#/TypeName" -> "#/$defs/TypeName"
			if (value.includes('.yaml#/')) {
				const typeName = value.split('#/')[1];
				result[key] = `#/$defs/${typeName}`;
			} else if (value.startsWith('#/') && !value.startsWith('#/$defs/')) {
				const typeName = value.substring(2);
				result[key] = `#/$defs/${typeName}`;
			} else {
				result[key] = value;
			}
		} else {
			result[key] = rewriteRefs(value);
		}
	}
	return result;
}

function loadSchemas(): Record<string, SchemaObject> {
	const defs: Record<string, SchemaObject> = {};

	for (const file of SCHEMA_FILES) {
		const filePath = path.join(SCHEMA_DIR, file);
		const content = fs.readFileSync(filePath, 'utf-8');
		const parsed = yaml.load(content) as Record<string, SchemaObject>;

		for (const [typeName, schema] of Object.entries(parsed)) {
			defs[typeName] = rewriteRefs(schema) as SchemaObject;
		}
	}

	return defs;
}

export interface SchemaValidators {
	validateRender(msg: unknown): boolean;
	validatePatch(msg: unknown): boolean;
	validateHello(msg: unknown): boolean;
	validateAction(msg: unknown): boolean;
	getErrors(): string | null;
}

/**
 * Create a validator that can check protocol messages against the OpenAPI schemas.
 */
export function createValidator(): SchemaValidators {
	const defs = loadSchemas();

	const ajv = new Ajv({ allErrors: true, strict: false });
	addFormats(ajv);

	// Build a meta-schema containing all definitions under $defs
	const metaSchema = {
		$id: 'marionette-protocol',
		$defs: defs,
	};

	ajv.addSchema(metaSchema);

	let lastErrors: string | null = null;

	function validate(schemaName: string, msg: unknown): boolean {
		// Create an inline schema that references the definition
		const schema = { $ref: `marionette-protocol#/$defs/${schemaName}` };
		const valid = ajv.validate(schema, msg);
		if (!valid) {
			lastErrors = ajv.errorsText(ajv.errors);
		} else {
			lastErrors = null;
		}
		return valid as boolean;
	}

	return {
		validateRender: (msg: unknown) => validate('RenderMessage', msg),
		validatePatch: (msg: unknown) => validate('PatchMessage', msg),
		validateHello: (msg: unknown) => validate('HelloMessage', msg),
		validateAction: (msg: unknown) => validate('ActionMessage', msg),
		getErrors: () => lastErrors,
	};
}
