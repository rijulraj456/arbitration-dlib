// Dispatcher provides the infrastructure to support the development of DApps,
// mediating the communication between on-chain and off-chain components. 

// Copyright (C) 2019 Cartesi Pte. Ltd.

// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.

// This program is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
// PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Note: This component currently has dependencies that are licensed under the GNU
// GPL, version 3, and so you should treat this component as a whole as being under
// the GPL version 3. But all Cartesi-written code in this component is licensed
// under the Apache License, version 2, or a compatible permissive license, and can
// be used independently under the Apache v2 license. After this component is
// rewritten, the entire component will be released under the Apache v2 license.



//! A collection of types that represent the manager grpc interface
//! together with the conversion functions from the automatically
//! generated types.

use super::ethereum_types::H256;
use super::{cartesi_base, manager_high};
use super::grpc::marshall::Marshaller;

pub const EMULATOR_SERVICE_NAME: &'static str = "emulator";
pub const EMULATOR_METHOD_NEW: &'static str = "/CartesiManagerHigh.MachineManagerHigh/NewSession";
pub const EMULATOR_METHOD_RUN: &'static str = "/CartesiManagerHigh.MachineManagerHigh/SessionRun";
pub const EMULATOR_METHOD_STEP: &'static str = "/CartesiManagerHigh.MachineManagerHigh/SessionStep";
pub const EMULATOR_METHOD_READ: &'static str = "/CartesiManagerHigh.MachineManagerHigh/SessionReadMemory";
pub const EMULATOR_METHOD_WRITE: &'static str = "/CartesiManagerHigh.MachineManagerHigh/SessionWriteMemory";
pub const EMULATOR_METHOD_PROOF: &'static str = "/CartesiManagerHigh.MachineManagerHigh/SessionGetProof";

/// Representation of a request for new session
#[derive(Debug, Clone)]
pub struct NewSessionRequest {
    pub machine: cartesi_base::MachineRequest,
    pub session_id: String,
}

impl From<manager_high::NewSessionRequest>
    for NewSessionRequest
{
    fn from(
        result: manager_high::NewSessionRequest,
    ) -> Self {
        NewSessionRequest {
            machine: result.machine
                    .into_option()
                    .expect("machine not found")
                    .into(),
            session_id: result.session_id,
        }
    }
}

/// Representation of a request for running the machine
#[derive(Debug, Clone)]
pub struct SessionRunRequest {
    pub session_id: String,
    pub times: Vec<u64>,
}

impl From<manager_high::SessionRunRequest>
    for SessionRunRequest
{
    fn from(
        result: manager_high::SessionRunRequest,
    ) -> Self {
        SessionRunRequest {
            session_id: result.session_id,
            times: result.final_cycles,
        }
    }
}

/// Representation of the result of running the machine
#[derive(Debug, Clone)]
pub struct SessionRunResult {
    pub hashes: Vec<H256>,
}

impl From<manager_high::SessionRunResult>
    for SessionRunResult
{
    fn from(
        result: manager_high::SessionRunResult,
    ) -> Self {
        SessionRunResult {
            hashes: result
                .hashes
                .into_vec()
                .into_iter()
                .map(|hash| H256::from_slice(&hash.content))
                .collect(),
        }
    }
}

/// Representation of the result of creating a new machine
#[derive(Debug, Clone)]
pub struct NewSessionResult {
    pub hash: H256,
}

impl From<cartesi_base::Hash>
    for NewSessionResult
{
    fn from(
        result: cartesi_base::Hash,
    ) -> Self {
        NewSessionResult {
            hash: H256::from_slice(&result.content)
        }
    }
}

/// Access operation is either a `Read` or a `Write`
#[derive(Debug, Clone)]
pub enum AccessOperation {
    Read,
    Write,
}

impl From<cartesi_base::AccessOperation> for AccessOperation {
    fn from(op: cartesi_base::AccessOperation) -> Self {
        match op {
            cartesi_base::AccessOperation::READ => AccessOperation::Read,
            cartesi_base::AccessOperation::WRITE => AccessOperation::Write,
        }
    }
}

/// A proof that a certain subtree has the contents represented by
/// `target_hash`.
#[derive(Debug, Clone)]
pub struct Proof {
    pub address: u64,
    pub log2_size: u32,
    pub target_hash: H256,
    pub sibling_hashes: Vec<H256>,
    pub root_hash: H256,
}

impl From<cartesi_base::Proof> for Proof {
    fn from(proof: cartesi_base::Proof) -> Self {
        Proof {
            address: proof.address,
            log2_size: proof.log2_size,
            target_hash: H256::from_slice(
                &proof
                    .target_hash
                    .into_option()
                    .expect("target hash not found")
                    .content,
            ),
            sibling_hashes: proof
                .sibling_hashes
                .into_vec()
                .into_iter()
                .map(|hash| H256::from_slice(&hash.content))
                .collect(),
            root_hash: H256::from_slice(
                &proof
                    .root_hash
                    .into_option()
                    .expect("root hash not found")
                    .content,
            ),
        }
    }
}

/// An access to be logged during the step procedure
#[derive(Debug, Clone)]
pub struct Access {
    pub operation: AccessOperation,
    pub address: u64,
    pub value_read: [u8; 8],
    pub value_written: [u8; 8],
    pub proof: Proof,
}

fn to_bytes(input: Vec<u8>) -> Option<[u8; 8]> {
    if input.len() != 8 {
        None
    } else {
        Some([
            input[0], input[1], input[2], input[3], input[4], input[5],
            input[6], input[7],
        ])
    }
}

impl From<cartesi_base::Access> for Access {
    fn from(access: cartesi_base::Access) -> Self {
        let proof: Proof =
            access.proof.into_option().expect("proof not found").into();
        Access {
            operation: access.operation.into(),
            address: proof.address,
            value_read: to_bytes(
                access
                    .read
                    .into_option()
                    .expect("read access not found")
                    .content,
            )
            .expect("read value has the wrong size"),
            value_written: to_bytes(
                access
                    .written
                    .into_option()
                    .expect("write access not found")
                    .content,
            )
            .expect("write value has the wrong size"),
            proof: proof,
        }
    }
}

/// A representation of a request for a logged machine step
#[derive(Debug, Clone)]
pub struct SessionStepRequest {
    pub session_id: String,
    pub time: u64,
}

impl From<manager_high::SessionStepRequest>
    for SessionStepRequest
{
    fn from(
        result: manager_high::SessionStepRequest,
    ) -> Self {
        SessionStepRequest {
            session_id: result.session_id,
            time: result.initial_cycle,
        }
    }
}

/// A representation of the result of a logged machine step
#[derive(Debug, Clone)]
pub struct SessionStepResult {
    pub log: Vec<Access>,
}

impl From<manager_high::SessionStepResult>
    for SessionStepResult
{
    fn from(
        result: manager_high::SessionStepResult,
    ) -> Self {
        SessionStepResult {
            log: result
                .log
                .into_option()
                .expect("log not found")
                .accesses
                .into_vec()
                .into_iter()
                .map(|hash| hash.into())
                .collect(),
        }
    }
}

/// Representation of a request for read the memory
#[derive(Debug, Clone)]
pub struct SessionReadMemoryRequest {
    pub session_id: String,
    pub time: u64,
    pub position: cartesi_base::ReadMemoryRequest
}

impl From<manager_high::SessionReadMemoryRequest>
    for SessionReadMemoryRequest
{
    fn from(
        result: manager_high::SessionReadMemoryRequest,
    ) -> Self {
        SessionReadMemoryRequest {
            session_id: result.session_id,
            time: result.cycle,
            position: result.position.into_option().expect("position not found").into()
        }
    }
}

/// A result from the read memory procedure
#[derive(Debug, Clone)]
pub struct ReadMemoryResponse {
    pub data: Vec<u8>
}

impl From<cartesi_base::ReadMemoryResponse> for ReadMemoryResponse {
    fn from(read: cartesi_base::ReadMemoryResponse) -> Self {
        ReadMemoryResponse {
            data: read.data
        }
    }
}

/// Representation of a response for read the memory
#[derive(Debug, Clone)]
pub struct SessionReadMemoryResult {
    pub read_content: ReadMemoryResponse
}

impl From<manager_high::SessionReadMemoryResult>
    for SessionReadMemoryResult
{
    fn from(
        result: manager_high::SessionReadMemoryResult,
    ) -> Self {
        SessionReadMemoryResult {
            read_content: result.read_content.into_option().expect("read_content not found").into()
        }
    }
}

/// Representation of a request for get proof
#[derive(Debug, Clone)]
pub struct SessionGetProofRequest {
    pub session_id: String,
    pub time: u64,
    pub target: cartesi_base::GetProofRequest
}

impl From<manager_high::SessionGetProofRequest>
    for SessionGetProofRequest
{
    fn from(
        result: manager_high::SessionGetProofRequest,
    ) -> Self {
        SessionGetProofRequest {
            session_id: result.session_id,
            time: result.cycle,
            target: result.target.into_option().expect("target not found").into()
        }
    }
}

/// Representation of a response for read the memory
#[derive(Debug, Clone)]
pub struct SessionGetProofResult {
    pub proof: Proof
}

impl From<cartesi_base::Proof>
    for SessionGetProofResult
{
    fn from(
        proof: cartesi_base::Proof,
    ) -> Self {
        SessionGetProofResult {
            proof: proof.into()
        }
    }
}

impl From<Vec<u8>>
    for SessionRunResult
{
    fn from(
        response: Vec<u8>,
    ) -> Self {
        let marshaller: Box<dyn Marshaller<manager_high::SessionRunResult> + Sync + Send> = Box::new(grpc::protobuf::MarshallerProtobuf);
        marshaller.read(bytes::Bytes::from(response)).unwrap().into()
    }
}

impl From<Vec<u8>>
    for SessionStepResult
{
    fn from(
        response: Vec<u8>,
    ) -> Self {
        let marshaller: Box<dyn Marshaller<manager_high::SessionStepResult> + Sync + Send> = Box::new(grpc::protobuf::MarshallerProtobuf);
        marshaller.read(bytes::Bytes::from(response)).unwrap().into()
    }
}

impl From<Vec<u8>>
    for NewSessionResult
{
    fn from(
        response: Vec<u8>,
    ) -> Self {
        let marshaller: Box<dyn Marshaller<cartesi_base::Hash> + Sync + Send> = Box::new(grpc::protobuf::MarshallerProtobuf);
        marshaller.read(bytes::Bytes::from(response)).unwrap().into()
    }
}

impl From<Vec<u8>>
    for SessionReadMemoryResult
{
    fn from(
        response: Vec<u8>,
    ) -> Self {
        let marshaller: Box<dyn Marshaller<manager_high::SessionReadMemoryResult> + Sync + Send> = Box::new(grpc::protobuf::MarshallerProtobuf);
        marshaller.read(bytes::Bytes::from(response)).unwrap().into()
    }
}

impl From<Vec<u8>>
    for SessionGetProofResult
{
    fn from(
        response: Vec<u8>,
    ) -> Self {
        let marshaller: Box<dyn Marshaller<cartesi_base::Proof> + Sync + Send> = Box::new(grpc::protobuf::MarshallerProtobuf);
        marshaller.read(bytes::Bytes::from(response)).unwrap().into()
    }
}

impl From<Vec<u8>>
    for NewSessionRequest
{
    fn from(
        response: Vec<u8>,
    ) -> Self {
        let marshaller: Box<dyn Marshaller<manager_high::NewSessionRequest> + Sync + Send> = Box::new(grpc::protobuf::MarshallerProtobuf);
        marshaller.read(bytes::Bytes::from(response)).unwrap().into()
    }
}

impl From<SessionRunRequest>
    for Vec<u8>
{
    fn from(
        request: SessionRunRequest,
    ) -> Self {
        let marshaller: Box<dyn Marshaller<manager_high::SessionRunRequest> + Sync + Send> = Box::new(grpc::protobuf::MarshallerProtobuf);
    
        let mut req = manager_high::SessionRunRequest::new();
        req.set_session_id(request.session_id);
        req.set_final_cycles(request.times);

        marshaller.write(&req).unwrap()
    }
}

impl From<SessionStepRequest>
    for Vec<u8>
{
    fn from(
        request: SessionStepRequest,
    ) -> Self {
        let marshaller: Box<dyn Marshaller<manager_high::SessionStepRequest> + Sync + Send> = Box::new(grpc::protobuf::MarshallerProtobuf);
    
        let mut req = manager_high::SessionStepRequest::new();
        req.set_session_id(request.session_id);
        req.set_initial_cycle(request.time);

        marshaller.write(&req).unwrap()
    }
}

impl From<NewSessionRequest>
    for Vec<u8>
{
    fn from(
        request: NewSessionRequest,
    ) -> Self {
        let marshaller: Box<dyn Marshaller<manager_high::NewSessionRequest> + Sync + Send> = Box::new(grpc::protobuf::MarshallerProtobuf);
    
        let mut req = manager_high::NewSessionRequest::new();
        req.set_session_id(request.session_id);
        req.set_machine(request.machine);

        marshaller.write(&req).unwrap()
    }
}

impl From<SessionReadMemoryRequest>
    for Vec<u8>
{
    fn from(
        request: SessionReadMemoryRequest,
    ) -> Self {
        let marshaller: Box<dyn Marshaller<manager_high::SessionReadMemoryRequest> + Sync + Send> = Box::new(grpc::protobuf::MarshallerProtobuf);
    
        let mut req = manager_high::SessionReadMemoryRequest::new();
        req.set_session_id(request.session_id);
        req.set_cycle(request.time);
        req.set_position(request.position);

        marshaller.write(&req).unwrap()
    }
}

impl From<SessionGetProofRequest>
    for Vec<u8>
{
    fn from(
        request: SessionGetProofRequest,
    ) -> Self {
        let marshaller: Box<dyn Marshaller<manager_high::SessionGetProofRequest> + Sync + Send> = Box::new(grpc::protobuf::MarshallerProtobuf);
    
        let mut req = manager_high::SessionGetProofRequest::new();
        req.set_session_id(request.session_id);
        req.set_cycle(request.time);
        req.set_target(request.target);

        marshaller.write(&req).unwrap()
    }
}
