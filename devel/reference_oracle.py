#!/usr/bin/env python3
"""Regenerate isolated original and deterministic Illustrate reference evidence."""

import json
import math
import shlex
import shutil
import hashlib
import pathlib
import difflib
import subprocess


#============================================
def run(command: list[str], directory: pathlib.Path) -> str:
	"""Run argument vectors directly (ASVS 1.2.5: no shell interpolation)."""
	result = subprocess.run(command, cwd=directory, check=True, capture_output=True, text=True)
	return result.stdout + result.stderr


#============================================
def digest(path: pathlib.Path) -> str:
	return hashlib.sha256(path.read_bytes()).hexdigest()


#============================================
def replace_once(source: str, before: str, after: str) -> str:
	"""Fail visibly if the pinned source layout changes."""
	if source.count(before) != 1:
		raise ValueError(f'Expected one source anchor: {before!r}')
	return source.replace(before, after)


#============================================
def deterministic_source(source: str) -> str:
	"""Apply only recorded initialization, flat fog, and diagnostic changes."""
	source = ('C Modified development oracle: initialized buffers and scratch,\n'
		'C guarded flat fog depth, and exported clamped depth/atom diagnostics.\n'
		'C Upstream rendering formulas otherwise retained; see deterministic.patch.\n' + source)
	source = replace_once(source, 'type(350000),res', 'type(0:350000),res')
	source = replace_once(source, '\trscale=1.',
		'\ttype(0)=0\n\tradius_max=0.\n\tautocenter=0.\n'
		'\txtranc=0.\n\tytranc=0.\n\tztranc=0.\n\tchainlast=" "\n\trscale=1.')
	source = replace_once(source, '\tdo ix=1,ixsize\n\tdo iy=1,iysize\n \t  pix',
		'\tdo ix=-10,3008\n\tdo iy=-10,3008\n \t  pix')
	source = replace_once(source, '\tzpix_spread=zpix_max-zpix_min',
		'\tzpix_spread=max(zpix_max-zpix_min,1.e-20)')
	source = replace_once(source, '\tzpix_max=min(zpix_max,0.)',
		'\twrite(6,*) "oracle raw z range",zpix_min,zpix_max\n'
		'\tzpix_max=min(zpix_max,0.)')
	source = replace_once(source, '\tg_opacity=0.\n\tl_opacity=0.',
		'\tg_opacity=0.\n\tr_opacity=0.\n\tl_opacity=0.')
	source = replace_once(source, '\tiyc=iy+iyl\n',
		'\tiyc=iy+iyl\n\tl(ixl,iyl)=0.\n')
	# Export after clipping, before shading; row-major order matches PPM output.
	source = replace_once(source, ' 1009\tenddo\n',
		' 1009\tenddo\n\topen(10,file="depth_atom.tsv")\n'
		'\twrite(10,*) iysize,ixsize\n\tdo ix=1,ixsize\n'
		'\tdo iy=1,iysize\n\twrite(10,*) zpix(ix,iy),atom(ix,iy)\n'
		'\tenddo\n\tenddo\n\tclose(10)\n')
	return source


#============================================
def scene_pdb() -> str:
	"""Asymmetric overlap, separated chains/residues and depth variation."""
	atoms = [('A', 1, -2., -2., -4.), ('B', 30, 0., 0., -3.),
		('B', 1, 2., 1., -1.), ('C', 60, -2., 3., -6.)]
	lines = []
	for index, (chain, residue, x, y, z) in enumerate(atoms, 1):
		lines.append(f'ATOM  {index:5d}  C   GLY {chain}{residue:4d}    '
			f'{x:8.3f}{y:8.3f}{z:8.3f}  1.00 20.00           C')
	return '\n'.join(lines) + '\nEND\n'


#============================================
def scene_input(contour: str, subunit: str, residue: str,
	shadow: int = 0, fog: str = '1.,1.') -> str:
	return ('read\nscene.pdb\nATOM  ---------- 0,9999, 0.8,0.5,0.3, 1.8\nEND\n'
		'center\nauto\nscale\n6.0\nwor\n'
		f'1.,1.,1.,1.,1.,1.,{fog}\n{shadow},0.01,2.,1.,0.2\n96,112\n'
		f'illustrate\n{contour}\n{subunit}\n{residue}\ncalculate\nimage.pnm\n')


#============================================
def ppm_info(path: pathlib.Path) -> dict:
	"""Validate complete P3 output without adding an image dependency."""
	fields = path.read_text().split()
	width, height, maximum = map(int, fields[1:4])
	pixels = list(map(int, fields[4:]))
	if fields[0] != 'P3' or maximum != 255 or len(pixels) != width * height * 3:
		raise ValueError(f'Incomplete PPM: {path}')
	if min(pixels) < 0 or max(pixels) > 255:
		raise ValueError(f'Invalid channel range: {path}')
	return {'width': width, 'height': height, 'sha256': digest(path)}


#============================================
def depth_info(path: pathlib.Path, width: int, height: int) -> dict:
	"""Check exported geometry and record its foreground extent."""
	lines = path.read_text().splitlines()
	if list(map(int, lines[0].split())) != [width, height] or len(lines) != width * height + 1:
		raise ValueError(f'Incomplete geometry export: {path}')
	foreground = []
	for line in lines[1:]:
		depth, atom = line.split()
		z, atom_id = float(depth), int(atom)
		if not math.isfinite(z) or z > 0 or atom_id < 0 or atom_id > 350000:
			raise ValueError(f'Invalid geometry export: {path}')
		if atom_id:
			foreground.append(z)
	if not foreground:
		raise ValueError(f'Expected occupied pixels: {path}')
	return {'occupied_pixels': len(foreground), 'minimum_depth': min(foreground),
		'maximum_depth': max(foreground)}


#============================================
def main() -> None:
	root = pathlib.Path(run(['git', 'rev-parse', '--show-toplevel'], pathlib.Path.cwd()).strip())
	reference = root / 'OTHER_REPOS/Illustrate'
	output = root / 'output_oracle'
	output.mkdir(exist_ok=True)
	source_path = reference / 'illustrate.f'
	source = source_path.read_text()
	patched = deterministic_source(source)
	(output / 'illustrate_deterministic.f').write_text(patched)
	(output / 'deterministic.patch').write_text(''.join(difflib.unified_diff(
		source.splitlines(True), patched.splitlines(True), 'illustrate.f', 'illustrate_deterministic.f')))
	compiler = shutil.which('gfortran')
	if compiler is None:
		raise RuntimeError('gfortran is required only to regenerate the development oracle')
	manifest = {'compiler': run([compiler, '--version'], root).splitlines()[0],
		'upstream_revision': run(['git', 'rev-parse', 'HEAD'], reference).strip(),
		'upstream_remote': run(['git', 'remote', 'get-url', 'origin'], reference).strip(),
		'source_sha256': digest(source_path), 'deterministic_sha256':
		digest(output / 'illustrate_deterministic.f'), 'commands': [], 'renders': []}
	for variant, input_source in [('original', source_path),
		('deterministic', output / 'illustrate_deterministic.f')]:
		command = [compiler, '-O0', '-std=legacy', str(input_source), '-o', str(output / f'illustrate_{variant}')]
		manifest['commands'].append(shlex.join(command))
		(output / f'{variant}_compile.log').write_text(run(command, root))
	disabled = '100000.,100001.'
	scenes = {'plain': None, 'shadow': None, 'fog': None,
		'subunit': None, 'residue': None}
	for kernel in range(1, 5):
		scenes[f'contour_{kernel}'] = scene_input(f'3.,10.,{kernel},0.,5.', disabled,
			f'{disabled},6000.')
	scenes['plain'] = scene_input(f'{disabled},4,0.,5.', disabled, f'{disabled},6000.')
	scenes['shadow'] = scene_input(f'{disabled},4,0.,5.', disabled,
		f'{disabled},6000.', shadow=1)
	scenes['fog'] = scene_input(f'{disabled},4,0.,5.', disabled,
		f'{disabled},6000.', fog='1.,0.2')
	scenes['subunit'] = scene_input(f'{disabled},4,0.,5.', '3.,10.', f'{disabled},6000.')
	scenes['residue'] = scene_input(f'{disabled},4,0.,5.', disabled, '3.,8.,10.')
	scenes['combined'] = scene_input('3.,10.,4,0.,5.', '3.,10.', '3.,8.,10.',
		shadow=1, fog='1.,0.2')
	scenes['flat_depth'] = scenes['plain'].replace('0.3, 1.8', '0.3, 0.1')
	scenes['clipped'] = scenes['combined'].replace('center\nauto\n',
		'center\ncen\ntrans\n0.,0.,-1.\n')
	scenes['edge'] = scenes['combined'].replace('center\nauto\n',
		'center\nauto\ntrans\n5.,7.,0.\n')
	scenes['hemoglobin'] = (reference / '2hhb.inp').read_text().replace('2hhb.pnm', 'image.pnm')
	for variant in ('original', 'deterministic'):
		for name, controls in scenes.items():
			directory = output / variant / name
			# Executables use separate names so their scene directory can stay predictable.
			directory.mkdir(parents=True, exist_ok=True)
			(directory / 'commands.inp').write_text(controls)
			if name == 'hemoglobin':
				shutil.copyfile(reference / '2hhb.pdb', directory / '2hhb.pdb')
			else:
				pdb = scene_pdb()
				if name == 'flat_depth':
					pdb = pdb.splitlines()[0] + '\nEND\n'
				(directory / 'scene.pdb').write_text(pdb)
			previous = None
			for repetition in range(2):
				with (directory / 'commands.inp').open() as stream:
					result = subprocess.run([str(output / f'illustrate_{variant}')],
						cwd=directory, stdin=stream, capture_output=True, text=True, check=True)
				(directory / f'run_{repetition + 1}.log').write_text(result.stdout + result.stderr)
				info = ppm_info(directory / 'image.pnm')
				current = [info['sha256'], digest(directory / 'opacity.pnm')]
				if variant == 'deterministic':
					current.append(digest(directory / 'depth_atom.tsv'))
				if previous is not None and current != previous:
					raise RuntimeError(f'Nonrepeatable render: {variant}/{name}')
				previous = current
			command_text = f'cd {directory.relative_to(root)} && ../../illustrate_{variant} < commands.inp'
			manifest['commands'].append(command_text)
			geometry = None
			if variant == 'deterministic':
				geometry = depth_info(directory / 'depth_atom.tsv', info['width'], info['height'])
			manifest['renders'].append({'variant': variant, 'scene': name, **info,
				'opacity_sha256': current[1], 'repeat_identical': True,
				'controls_sha256': digest(directory / 'commands.inp'),
				'input_sha256': digest(directory / ('2hhb.pdb' if name == 'hemoglobin'
					else 'scene.pdb')),
				'depth_atom_sha256': current[2] if variant == 'deterministic' else None,
				'geometry': geometry})
	(output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
	print(f'Validated {len(manifest["renders"])} repeatable renders: {output}')


if __name__ == '__main__':
	main()
