import {
  BufferGeometry,
  Float32BufferAttribute,
  Points,
  PointsMaterial,
} from "three";

class Potree {
  constructor() {}

  parse(data, metadata) {
    // build geometry
    const intBuffer = new Int32Array(data);
    const scale = metadata.scale;
    const offset = metadata.offset;
    const numPoints = intBuffer.byteLength / (4 * 3);
    const positions = new Float32Array(numPoints * 3);

    let i = 0;
    for (let num of intBuffer) {
      positions[i] = num * scale[i % 3] + offset[i % 3];

      i++;
    }

    const geometry = new BufferGeometry();
    geometry.setAttribute("position", new Float32BufferAttribute(positions, 3));
    geometry.computeBoundingSphere();

    const material = new PointsMaterial({ size: 0.05 });
    material.color.setHex(0x00ff00);

    return new Points(geometry, material);
  }
}

export { Potree };
