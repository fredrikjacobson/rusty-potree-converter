import * as wasm from "potree-converter";
import {
  Scene,
	Vector3,
  Points,
  BufferGeometry,
  Float32BufferAttribute,
  PointsMaterial,
} from "three";
import * as THREE from "three";
import { Potree } from "./PotreeLoader";

const scene = new Scene();
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
const camera = new THREE.PerspectiveCamera(
  75,
  window.innerWidth / window.innerHeight,
  0.1,
  1000
);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);
const controls = new OrbitControls(camera, renderer.domElement);
camera.position.set(0, -25, 25);
camera.lookAt(new Vector3());
// controls.minDistance = 3;
// controls.maxDistance = 20;
controls.update();

function animate() {
  requestAnimationFrame(animate);
  renderer.render(scene, camera);
}
animate();

const output = document.getElementById("potree-metadata");

if (window.FileList && window.File) {
  const dropArea = document.body;
  dropArea.addEventListener("dragover", (event) => {
    event.stopPropagation();
    event.preventDefault();
    // Style the drag-and-drop as a "copy file" operation.
    event.dataTransfer.dropEffect = "copy";
  });

  dropArea.addEventListener("drop", (event) => {
    event.stopPropagation();
    event.preventDefault();

    if (event.dataTransfer.files.length > 0) {
      const file = event.dataTransfer.files[0];
      file.arrayBuffer().then((buf) => {
        let potree = wasm.process_array_buffer("pcd", new Uint8Array(buf));
        const metadataInfo = document.createElement("pre");
        metadataInfo.style.color = "white";
        metadataInfo.style.fontSize = 6;
        metadataInfo.textContent = JSON.stringify(
          potree.get_metadata(),
          undefined,
          4
        );
        // output.appendChild(metadataInfo);

        let potree_obj = new Potree().parse(potree.get_octree().buffer, potree.get_metadata());
        scene.add(potree_obj);
      });
    }
  });
}
