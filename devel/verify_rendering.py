#!/usr/bin/env python3
"""Compare native production passes with the deterministic development oracle."""

import re
import json
import gzip
import pathlib
import hashlib
import subprocess

import PIL.Image
import PIL.ImageDraw
import PIL.ImageChops
import PIL.ImageEnhance


#============================================
def digest(path: pathlib.Path) -> str:
	return hashlib.sha256(path.read_bytes()).hexdigest()


#============================================
def scene_settings(reference: pathlib.Path, root: pathlib.Path) -> dict:
	"""Translate the retained source cards, preserving their explicit camera."""
	name = reference.name
	lines = (reference / 'commands.inp').read_text().splitlines()
	log = (reference / 'run_1.log').read_text()
	center = [float(value) for value in re.search(
		r'centering vector\s*:\s*([^\n]+)', log)[1].split()]
	image = PIL.Image.open(reference / 'image.pnm')
	if name.startswith('hemoglobin'):
		settings = json.loads((root / 'examples/hemoglobin.json').read_text())
	else:
		illustrate = lines.index('illustrate')
		contour = list(map(float, lines[illustrate + 1].split(',')))
		subunit = list(map(float, lines[illustrate + 2].split(',')))
		residue = list(map(float, lines[illustrate + 3].split(',')))
		world = lines.index('wor')
		fog = list(map(float, lines[world + 1].split(',')))
		shadow = list(map(float, lines[world + 2].split(',')))
		settings = {'atom_rules': [{'selector': {}, 'color': [.8, .5, .3],
			'radius': .1 if name == 'flat_depth' else 1.8}],
			'fog': {'color': fog[3:6], 'front': fog[6], 'back': fog[7]},
			'shadows': {'enabled': bool(shadow[0]), 'decrement': shadow[1],
				'slope': shadow[2], 'threshold': shadow[3], 'brightness_floor': shadow[4]},
			'outlines': {'contour_enabled': True, 'subunit_enabled': True,
				'residue_enabled': True, 'kernel': int(contour[2]),
				'contour': {'low': contour[0], 'high': contour[1]},
				'subunit': {'low': subunit[0], 'high': subunit[1]},
				'residue': {'low': residue[0], 'high': residue[1]},
				'depth_min': contour[3], 'depth_max': contour[4],
				'residue_difference': residue[2]}}
		if 'trans' in lines:
			settings['translation'] = list(map(float, lines[lines.index('trans') + 1].split(',')))
	settings['width'], settings['height'] = image.size
	# Reference cards own the oracle background; gallery scenes inherit transparency.
	settings['background'] = [float(value) for value in
		lines[lines.index('wor') + 1].split(',')[:3]]
	if name.startswith('hemoglobin_'):
		part = name.removeprefix('hemoglobin_')
		settings['outlines']['contour_enabled'] = part == 'contour'
		settings['outlines']['subunit_enabled'] = part == 'subunit'
		settings['outlines']['residue_enabled'] = part == 'residue'
		settings['shadows']['enabled'] = part == 'shadow'
	settings['framing'] = {'center': [-value for value in center],
		'scale': float(lines[lines.index('scale') + 1]), 'margin': 0}
	return settings


#============================================
def compare(reference: pathlib.Path, native: pathlib.Path) -> dict:
	"""Measure exact bytes and geometry independently of lossy evidence views."""
	original = PIL.Image.open(reference / 'image.pnm').convert('RGB')
	actual = PIL.Image.open(native / 'image.png').convert('RGB')
	if actual.size != original.size:
		raise ValueError(f'Dimension mismatch: {native}')
	difference = PIL.ImageChops.difference(original, actual)
	histogram = difference.histogram()
	error_sum = sum(index % 256 * count for index, count in enumerate(histogram))
	different_pixels = sum(any(pixel) for pixel in zip(*[iter(difference.tobytes())] * 3))
	rows = []
	for path in [reference / 'depth_atom.tsv', native / 'depth_atom.tsv']:
		rows.append([line.split() for line in path.read_text().splitlines()[1:]])
	if len(rows[0]) != actual.width * actual.height or len(rows[1]) != len(rows[0]):
		raise ValueError('Incomplete depth map')
	mask_differences = sum((int(a[1]) == 0) != (int(b[1]) == 0)
		for a, b in zip(*rows))
	owners = [int(line.split()[0]) for line in (native / 'sphere_source.tsv').read_text().splitlines()]
	if reference.name.startswith('hemoglobin'):
		pdb_lines = (reference / '2hhb.pdb').read_text().splitlines()
		atoms = [line for line in pdb_lines if line.startswith(('ATOM  ', 'HETATM'))]
		retained = [index for index, line in enumerate(atoms)
			if (line.startswith('ATOM  ') and not line[12:16].strip().startswith(('H', 'D')))
			or (line.startswith('HETATM') and line[17:20] == 'HEM')]
	else:
		retained = list(range(len(owners)))
	owner_differences = sum(
		(retained[int(a[1]) - 1] if int(a[1]) else None)
		!= (owners[int(b[1]) - 1] if int(b[1]) else None) for a, b in zip(*rows))
	depth_errors = [abs(float(a[0]) - float(b[0])) for a, b in zip(*rows)
		if int(a[1]) and int(b[1])]
	original.save(native / 'oracle.webp', quality=90)
	actual.save(native / 'native.webp', quality=90)
	PIL.ImageEnhance.Brightness(difference).enhance(8).save(native / 'difference_x8.webp', quality=90)
	width, height = original.size
	sheet = PIL.Image.new('RGB', (width * 3, height + 24), 'white')
	draw = PIL.ImageDraw.Draw(sheet)
	for index, (label, image) in enumerate([
		('Oracle', original), ('Native', actual), ('Difference x8', PIL.ImageEnhance.Brightness(difference).enhance(8))]):
		sheet.paste(image, (index * width, 24))
		draw.text((index * width + 4, 4), label, fill='black')
	sheet.save(native / 'comparison.webp', quality=90)
	return {'size': list(actual.size), 'different_pixels': different_pixels,
		'mean_absolute_channel_error': error_sum / (width * height * 3),
		'maximum_channel_error': max(difference.getextrema()[channel][1] for channel in range(3)),
		'mask_differences': mask_differences, 'owner_differences': owner_differences,
		'maximum_common_depth_error': max(depth_errors),
		'native_png_sha256': digest(native / 'image.png'),
		'oracle_ppm_sha256': digest(reference / 'image.pnm')}


#============================================
def main() -> None:
	root = pathlib.Path(__file__).resolve().parents[1]
	output = root / 'output_fidelity'
	output.mkdir(exist_ok=True)
	manifest = {'scenes': {}, 'commands': [], 'probe': 'examples/render_passes.rs'}
	samples = output / 'samples'
	samples.mkdir(exist_ok=True)
	manifest['sample_equivalence'] = {}
	first_pixels = None
	for extension in ['pdb', 'cif', 'xml']:
		plain = root / 'examples' / f'sample.{extension}'
		compressed = samples / f'sample.{extension}.gz'
		compressed.write_bytes(gzip.compress(plain.read_bytes(), mtime=0))
		for input_path in [plain, compressed]:
			image_path = samples / f'{input_path.name}.png'
			# ASVS 1.2.5: paths remain argument-vector values, never shell source.
			command = [str(root / 'target/release/illustrate'), str(input_path),
				'--output', str(image_path), '--width', '128', '--height', '128']
			subprocess.run(command, capture_output=True, check=True)
			pixels = PIL.Image.open(image_path).tobytes()
			if first_pixels is not None and pixels != first_pixels:
				raise ValueError(f'Sample format image differs: {input_path}')
			first_pixels = pixels
			manifest['sample_equivalence'][input_path.name] = digest(image_path)
			manifest['commands'].append(command)
	manifest['probe_sha256'] = digest(root / 'examples/render_passes.rs')
	manifest['production_sha256'] = {path.name: digest(path) for path in [
		root / 'src/raster.rs', root / 'src/shading.rs', root / 'src/outlines.rs',
		root / 'src/render.rs', root / 'src/scene_preparation.rs']}
	references = sorted((root / 'output_oracle/deterministic').iterdir())
	for reference in references:
		if not reference.is_dir():
			continue
		native = output / reference.name
		native.mkdir(exist_ok=True)
		settings = native / 'settings.json'
		settings.write_text(json.dumps(scene_settings(reference, root), indent=2) + '\n')
		input_path = reference / ('2hhb.pdb' if reference.name.startswith('hemoglobin') else 'scene.pdb')
		command = [str(root / 'target/release/examples/render_passes'), str(input_path),
			str(settings), str(native)]
		result = subprocess.run(command, check=True, capture_output=True, text=True)
		(native / 'probe.json').write_text(result.stdout)
		(native / 'probe.stderr').write_text(result.stderr)
		metrics = compare(reference, native)
		metrics['probe'] = json.loads(result.stdout)
		metrics['settings_sha256'] = digest(settings)
		metrics['input_sha256'] = digest(input_path)
		manifest['commands'].append(command)
		manifest['scenes'][reference.name] = metrics
		print(reference.name, json.dumps(metrics), flush=True)
	(output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
	names = [name for name in manifest['scenes'] if not name.startswith('hemoglobin')]
	sheet = PIL.Image.new('RGB', (450, len(names) * 120), 'white')
	draw = PIL.ImageDraw.Draw(sheet)
	for index, name in enumerate(names):
		draw.text((3, index * 120 + 4), name, fill='black')
		for column, filename in enumerate(['oracle.webp', 'native.webp', 'difference_x8.webp']):
			image = PIL.Image.open(output / name / filename)
			sheet.paste(image, (110 + column * 112, index * 120 + 20))
	sheet.save(output / 'spheres_contact.webp', quality=90)


if __name__ == '__main__':
	main()
