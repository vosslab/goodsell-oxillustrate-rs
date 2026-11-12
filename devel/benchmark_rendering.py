#!/usr/bin/env python3
"""Record local release timings, input provenance, and biological expansion counts."""

import re
import json
import time
import random
import pathlib
import hashlib
import subprocess
import urllib.request

import PIL.Image
import PIL.ImageDraw


#============================================
def main() -> None:
	root = pathlib.Path(__file__).resolve().parents[1]
	output = root / 'output_fidelity'
	inputs = output / 'inputs'
	inputs.mkdir(parents=True, exist_ok=True)
	manifest = {'hardware': {}, 'cases': {}}
	for command in [['uname', '-a'], ['sw_vers'], ['sysctl', '-n', 'machdep.cpu.brand_string'],
		['sysctl', '-n', 'hw.memsize'], ['sysctl', '-n', 'hw.ncpu'], ['rustc', '--version']]:
		result = subprocess.run(command, capture_output=True, text=True, check=False)
		manifest['hardware'][' '.join(command)] = {'status': result.returncode,
			'stdout': result.stdout.strip(), 'stderr': result.stderr.strip()}
	for identifier in ['1lmb', '1cwp']:
		path = inputs / f'{identifier}.cif'
		url = f'https://files.rcsb.org/download/{identifier.upper()}.cif'
		if not path.exists():
			time.sleep(random.random())
			with urllib.request.urlopen(url, timeout=60) as response:  # nosec B310: fixed HTTPS URL.
				path.write_bytes(response.read())
	for name, input_path, assembly in [
		('hemoglobin', root / 'output_oracle/deterministic/hemoglobin/2hhb.pdb', None),
		('1lmb', inputs / '1lmb.cif', None), ('1cwp_assembly1', inputs / '1cwp.cif', '1')]:
		case = output / 'benchmarks' / name
		case.mkdir(parents=True, exist_ok=True)
		settings = case / 'settings.json'
		settings.write_text(json.dumps({'width': 800, 'height': 800, 'assembly': assembly}) + '\n')
		# ASVS 1.2.5: paths remain argument-vector values, never shell source.
		command = [str(root / 'target/release/illustrate'), str(input_path), '--scene',
			str(settings), '--output', str(case / 'image.png')]
		if assembly is not None:
			command.extend(['--assembly', assembly])
		measurements = []
		for repeat in range(3):
			result = subprocess.run(['/usr/bin/time', '-l', *command], check=True,
				capture_output=True, text=True)
			(case / f'time_{repeat + 1}.txt').write_text(result.stderr)
			wall = float(re.search(r'([\d.]+) real', result.stderr)[1])
			rss = int(re.search(r'(\d+)\s+maximum resident set size', result.stderr)[1])
			measurements.append({'wall_seconds': wall, 'peak_rss_bytes': rss})
		probe_command = [str(root / 'target/release/examples/render_passes'), str(input_path),
			str(settings), str(case / 'passes')]
		probe = subprocess.run(probe_command, check=True, capture_output=True, text=True)
		(case / 'probe.json').write_text(probe.stdout)
		image = PIL.Image.open(case / 'image.png')
		image.save(case / 'image.webp', quality=90)
		manifest['cases'][name] = {'command': command, 'probe_command': probe_command,
			'input_sha256': hashlib.sha256(input_path.read_bytes()).hexdigest(),
			'input_url': f'https://files.rcsb.org/download/{name[:4].upper()}.cif'
				if name != 'hemoglobin' else 'retained upstream 2hhb.pdb; oracle manifest provenance',
			'measurements': measurements, 'probe': json.loads(probe.stdout),
			'image_sha256': hashlib.sha256((case / 'image.png').read_bytes()).hexdigest()}
		print(name, json.dumps(manifest['cases'][name]), flush=True)
	(output / 'benchmarks.json').write_text(json.dumps(manifest, indent=2) + '\n')
	sheet = PIL.Image.new('RGB', (1200, 425), 'white')
	draw = PIL.ImageDraw.Draw(sheet)
	for index, name in enumerate(manifest['cases']):
		image = PIL.Image.open(output / 'benchmarks' / name / 'image.png')
		image.thumbnail((400, 400))
		sheet.paste(image, (index * 400, 25))
		draw.text((index * 400 + 5, 5), name, fill='black')
	sheet.save(output / 'scale_contact.webp', quality=90)


if __name__ == '__main__':
	main()
