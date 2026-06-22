// 由 build.rs 生成的代码,re-export
pub mod google {
    pub mod protobuf {
        tonic::include_proto!("google.protobuf");
    }
}

pub mod gpai {
    pub mod common {
        pub mod v1 {
            tonic::include_proto!("gpai.common.v1");
        }
    }
    pub mod instrument {
        pub mod v1 {
            tonic::include_proto!("gpai.instrument.v1");
        }
    }
    pub mod market {
        pub mod v1 {
            tonic::include_proto!("gpai.market.v1");
        }
    }
    pub mod portfolio {
        pub mod v1 {
            tonic::include_proto!("gpai.portfolio.v1");
        }
    }
    pub mod ingestion {
        pub mod v1 {
            tonic::include_proto!("gpai.ingestion.v1");
        }
    }
}
