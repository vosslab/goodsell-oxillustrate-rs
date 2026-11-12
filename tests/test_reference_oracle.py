"""Protect accepted oracle evidence when regeneration fails."""

# Standard Library
import pathlib
import importlib.util
import unittest.mock

# PIP3 modules
import pytest


#============================================
def test_failed_regeneration_preserves_evidence(tmp_path: pathlib.Path) -> None:
	"""Keep accepted data and remove temporary source after partial generation."""
	root = pathlib.Path(__file__).resolve().parents[1]
	spec = importlib.util.spec_from_file_location('reference_oracle', root / 'devel/reference_oracle.py')
	oracle = importlib.util.module_from_spec(spec)
	spec.loader.exec_module(oracle)
	output = tmp_path / 'output'
	output.mkdir()
	(output / 'manifest.json').write_text('accepted provenance')
	(output / 'image.pnm').write_bytes(b'accepted image')
	workspaces = []

	def prepare(reference: pathlib.Path, workspace: pathlib.Path) -> list[dict]:
		workspaces.append(workspace)
		(workspace / 'temporary_source.txt').write_text('temporary input')
		return []

	def generate(root: pathlib.Path, target: pathlib.Path, reference: pathlib.Path,
		provenance: list[dict]) -> None:
		target.mkdir()
		(target / 'image.pnm').write_bytes(b'partial replacement')
		raise RuntimeError('generation failed')

	with unittest.mock.patch.object(oracle, 'prepare_reference', side_effect=prepare):
		with unittest.mock.patch.object(oracle, 'generate', side_effect=generate):
			with pytest.raises(RuntimeError, match='generation failed'):
				oracle.regenerate(root, output, tmp_path / 'reference')
	assert (output / 'manifest.json').read_text() == 'accepted provenance'
	assert (output / 'image.pnm').read_bytes() == b'accepted image'
	assert not workspaces[0].exists()
	assert list(tmp_path.iterdir()) == [output]
