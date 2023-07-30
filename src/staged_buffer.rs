use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

use wgpu::util::DeviceExt;

struct StagedChangeToBuffer {
    offset: wgpu::BufferAddress,
    size: wgpu::BufferAddress,
}

pub struct StagedBuffer {
    gpu_buffer: wgpu::Buffer,
    staging_buffer: wgpu::Buffer,
    changes: Arc<RwLock<Vec<StagedChangeToBuffer>>>
}

impl StagedBuffer {
    pub fn new(device: &wgpu::Device, desc: wgpu::util::BufferInitDescriptor) -> Self {
        let gpu_buffer_usages = desc.usage.difference( // Prevent CPU access to GPU memory
            wgpu::BufferUsages::COPY_SRC |
            wgpu::BufferUsages::MAP_WRITE |
            wgpu::BufferUsages::MAP_READ
        ).union(wgpu::BufferUsages::COPY_DST);
        let gpu_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                usage: gpu_buffer_usages,
                ..desc
            }
        );
        let staging_buffer_usages = wgpu::BufferUsages::COPY_DST |wgpu::BufferUsages::COPY_SRC;
        let staging_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                usage: staging_buffer_usages,
                ..desc
            }
        );
        StagedBuffer { gpu_buffer, staging_buffer, changes: Default::default() }
    }
    pub fn stage(&self, queue: &wgpu::Queue, offset: wgpu::BufferAddress, data: &[u8]) {
        queue.write_buffer(&self.staging_buffer, offset, data);
        let size = data.len();
        let mut changes = self.changes.write().unwrap();
        changes.push(StagedChangeToBuffer {
            offset,
            size: size as wgpu::BufferAddress
        });
    }
    pub fn gpu_copy(&self, encoder: &mut wgpu::CommandEncoder) {
        self.changes.write().unwrap().drain(..).for_each(|change| {
            // staging_buffer and gpu_buffer should be like parallel arrays
            encoder.copy_buffer_to_buffer(
                &self.staging_buffer, change.offset,
                &self.gpu_buffer, change.offset,
                change.size
            );
        });
    }
}

impl Deref for StagedBuffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.gpu_buffer
    }
}
