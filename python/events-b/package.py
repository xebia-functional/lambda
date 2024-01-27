"""
This script creates a zip file that can be uploaded to AWS Lambda.
"""

import os
import site
from zipfile import ZipFile, ZIP_DEFLATED

def zip_into(path: str, archive: ZipFile) -> None:
	"""
	Recursively zip the contents of a directory into a ZipFile.
	"""
	# pylint: disable-next=unused-variable
	for root, dirs, files in os.walk(path):
		for file in files:
			archive.write(
				os.path.join(root, file),
				os.path.relpath(
					os.path.join(root, file),
					os.path.join(path, '..')))

def create_lambda_package(out: str = "dist/events-b.zip") -> None:
	"""
	Create a zip file that can be uploaded to AWS Lambda.
	"""
	with ZipFile(out, 'w', ZIP_DEFLATED) as archive:
		for package in site.getsitepackages():
			for d in os.listdir(package):
				zip_into(os.path.join(package, d), archive)
		zip_into('datum', archive)
		archive.write('events_b.py')
		archive.write('py.typed')
		archive.close()

if __name__ == "__main__":
	create_lambda_package()
