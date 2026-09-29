"""Tests for dataset loaders: alibaba2018, alibaba2021, azure2019, borg2019."""

import polars as pl
import pytest
from ksim.datasets import alibaba2018, alibaba2021, azure2019, borg2019


class TestAlibaba2018:
    """Test alibaba2018 loader."""

    def test_alibaba2018_load_basic(self, tmp_path):
        """Load produces correct columns and computation."""
        meta_path = tmp_path / "container_meta.csv"
        usage_path = tmp_path / "container_usage.csv"

        meta_path.write_text("c1,m1,100,app1,ok,400,500,50\n")
        usage_path.write_text("c1,m1,0,50,40,0,0,0,0,0,0\n")

        result = alibaba2018.load(meta_path, usage_path)
        assert isinstance(result, pl.DataFrame)
        assert set(result.columns) >= {
            "time_s",
            "service",
            "instance",
            "cpu_millis",
            "memory_bytes",
            "request_rate",
        }
        assert len(result) == 1

    def test_alibaba2018_cpu_millis_computation(self, tmp_path):
        """cpu_millis = util/100 * cpu_request * 10."""
        meta_path = tmp_path / "container_meta.csv"
        usage_path = tmp_path / "container_usage.csv"

        meta_path.write_text("c1,m1,100,app1,ok,400,500,50\n")
        # util 50, request 400 → cpu = 50/100 * 400 * 10 = 2000
        usage_path.write_text("c1,m1,0,50,40,0,0,0,0,0,0\n")

        result = alibaba2018.load(meta_path, usage_path)
        assert result["cpu_millis"][0] == 2000.0

    def test_alibaba2018_memory_computation(self, tmp_path):
        """memory = mem_util/100 * mem_size * 2**30."""
        meta_path = tmp_path / "container_meta.csv"
        usage_path = tmp_path / "container_usage.csv"

        meta_path.write_text("c1,m1,100,app1,ok,400,500,50\n")
        # util 40, size 50 → memory = 40/100 * 50 * 2**30
        usage_path.write_text("c1,m1,0,50,40,0,0,0,0,0,0\n")

        result = alibaba2018.load(meta_path, usage_path)
        expected_mem = 0.4 * 50 * (2**30)
        assert result["memory_bytes"][0] == pytest.approx(expected_mem)

    def test_alibaba2018_request_rate_from_cpu(self, tmp_path):
        """request_rate = cpu_millis / 50."""
        meta_path = tmp_path / "container_meta.csv"
        usage_path = tmp_path / "container_usage.csv"

        meta_path.write_text("c1,m1,100,app1,ok,400,500,50\n")
        usage_path.write_text("c1,m1,0,50,40,0,0,0,0,0,0\n")

        result = alibaba2018.load(meta_path, usage_path)
        # cpu = 2000, rate = 2000 / 50 = 40
        assert result["request_rate"][0] == 40.0

    def test_alibaba2018_drops_util_over_100(self, tmp_path):
        """Utilization > 100 is dropped."""
        meta_path = tmp_path / "container_meta.csv"
        usage_path = tmp_path / "container_usage.csv"

        meta_path.write_text("c1,m1,100,app1,ok,400,500,50\n")
        usage_path.write_text("c1,m1,0,101,40,0,0,0,0,0,0\n")

        result = alibaba2018.load(meta_path, usage_path)
        assert len(result) == 0

    def test_alibaba2018_drops_rows_without_meta(self, tmp_path):
        """Rows without metadata are dropped."""
        meta_path = tmp_path / "container_meta.csv"
        usage_path = tmp_path / "container_usage.csv"

        meta_path.write_text("c1,m1,100,app1,ok,400,500,50\n")
        # c2 has no meta
        usage_path.write_text("c1,m1,0,50,40,0,0,0,0,0,0\nc2,m2,0,50,40,0,0,0,0,0,0\n")

        result = alibaba2018.load(meta_path, usage_path)
        assert len(result) == 1


class TestAlibaba2021:
    """Test alibaba2021 loader."""

    def test_alibaba2021_load_basic(self, tmp_path):
        """Load produces correct columns."""
        ms_resource = tmp_path / "MSResource_0.csv"
        ms_rt_qps = tmp_path / "MSRTQps_0.csv"

        ms_resource.write_text(
            "timestamp,msname,msinstanceid,nodeid,cpu_utilization,memory_utilization\n"
            "0,web,i1,n1,0.5,0.4\n"
        )
        ms_rt_qps.write_text(
            "timestamp,msname,msinstanceid,metric,value\n0,web,i1,providerRPC_MCR,100\n"
        )

        result = alibaba2021.load([ms_resource], [ms_rt_qps])
        assert isinstance(result, pl.DataFrame)
        assert set(result.columns) >= {
            "time_s",
            "service",
            "instance",
            "cpu_millis",
            "memory_bytes",
            "request_rate",
        }

    def test_alibaba2021_time_conversion(self, tmp_path):
        """time_s = timestamp / 1000."""
        ms_resource = tmp_path / "MSResource_0.csv"
        ms_rt_qps = tmp_path / "MSRTQps_0.csv"

        ms_resource.write_text(
            "timestamp,msname,msinstanceid,nodeid,cpu_utilization,memory_utilization\n"
            "5000,web,i1,n1,0.5,0.4\n"
        )
        ms_rt_qps.write_text(
            "timestamp,msname,msinstanceid,metric,value\n5000,web,i1,providerRPC_MCR,100\n"
        )

        result = alibaba2021.load([ms_resource], [ms_rt_qps])
        assert result["time_s"][0] == 5.0

    def test_alibaba2021_cpu_millis(self, tmp_path):
        """cpu_millis = cpu_utilization * container_cpu_millis."""
        ms_resource = tmp_path / "MSResource_0.csv"
        ms_rt_qps = tmp_path / "MSRTQps_0.csv"

        ms_resource.write_text(
            "timestamp,msname,msinstanceid,nodeid,cpu_utilization,memory_utilization\n"
            "0,web,i1,n1,0.5,0.4\n"
        )
        ms_rt_qps.write_text(
            "timestamp,msname,msinstanceid,metric,value\n0,web,i1,providerRPC_MCR,100\n"
        )

        result = alibaba2021.load([ms_resource], [ms_rt_qps])
        # cpu = 0.5 * 1000 = 500
        assert result["cpu_millis"][0] == 500.0

    def test_alibaba2021_rate_from_metrics(self, tmp_path):
        """request_rate = sum of providerRPC_MCR and HTTP_MCR."""
        ms_resource = tmp_path / "MSResource_0.csv"
        ms_rt_qps = tmp_path / "MSRTQps_0.csv"

        ms_resource.write_text(
            "timestamp,msname,msinstanceid,nodeid,cpu_utilization,memory_utilization\n"
            "0,web,i1,n1,0.5,0.4\n"
        )
        ms_rt_qps.write_text(
            "timestamp,msname,msinstanceid,metric,value\n"
            "0,web,i1,providerRPC_MCR,100\n"
            "0,web,i1,HTTP_MCR,50\n"
        )

        result = alibaba2021.load([ms_resource], [ms_rt_qps])
        assert result["request_rate"][0] == 150.0

    def test_alibaba2021_rate_zero_when_absent(self, tmp_path):
        """request_rate = 0 when no rate metrics."""
        ms_resource = tmp_path / "MSResource_0.csv"
        ms_rt_qps = tmp_path / "MSRTQps_0.csv"

        ms_resource.write_text(
            "timestamp,msname,msinstanceid,nodeid,cpu_utilization,memory_utilization\n"
            "0,web,i1,n1,0.5,0.4\n"
        )
        ms_rt_qps.write_text(
            "timestamp,msname,msinstanceid,metric,value\n0,web,i1,someOtherMetric,100\n"
        )

        result = alibaba2021.load([ms_resource], [ms_rt_qps])
        assert result["request_rate"][0] == 0.0


class TestAzure2019:
    """Test azure2019 loader."""

    def test_azure2019_load_day_basic(self, tmp_path):
        """Load day produces correct columns."""
        invocations = tmp_path / "invocations.csv"
        durations = tmp_path / "durations.csv"
        memory = tmp_path / "memory.csv"

        invocations.write_text("HashOwner,HashApp,HashFunction,Trigger,1\no1,app1,f1,http,60\n")
        durations.write_text(
            "HashOwner,HashApp,HashFunction,Average,Count,Minimum,Maximum\n"
            "o1,app1,f1,100,100,50,200\n"
        )
        memory.write_text("HashOwner,HashApp,SampleCount,AverageAllocatedMb\no1,app1,100,256\n")

        result = azure2019.load_day(1, invocations, durations, memory)
        assert isinstance(result, pl.DataFrame)
        assert set(result.columns) >= {
            "time_s",
            "service",
            "instance",
            "cpu_millis",
            "memory_bytes",
            "request_rate",
        }

    def test_azure2019_minute_timing(self, tmp_path):
        """Minute 1 at time 0, minute 2 at time 60."""
        invocations = tmp_path / "invocations.csv"
        durations = tmp_path / "durations.csv"
        memory = tmp_path / "memory.csv"

        invocations.write_text(
            "HashOwner,HashApp,HashFunction,Trigger,1,2\no1,app1,f1,http,60,120\n"
        )
        durations.write_text(
            "HashOwner,HashApp,HashFunction,Average,Count,Minimum,Maximum\n"
            "o1,app1,f1,100,100,50,200\n"
        )
        memory.write_text("HashOwner,HashApp,SampleCount,AverageAllocatedMb\no1,app1,100,256\n")

        result = azure2019.load_day(1, invocations, durations, memory)
        times = sorted(result["time_s"].to_list())
        assert times == [0.0, 60.0]

    def test_azure2019_rate_computation(self, tmp_path):
        """request_rate = invocations / 60."""
        invocations = tmp_path / "invocations.csv"
        durations = tmp_path / "durations.csv"
        memory = tmp_path / "memory.csv"

        invocations.write_text("HashOwner,HashApp,HashFunction,Trigger,1\no1,app1,f1,http,120\n")
        durations.write_text(
            "HashOwner,HashApp,HashFunction,Average,Count,Minimum,Maximum\n"
            "o1,app1,f1,100,100,50,200\n"
        )
        memory.write_text("HashOwner,HashApp,SampleCount,AverageAllocatedMb\no1,app1,100,256\n")

        result = azure2019.load_day(1, invocations, durations, memory)
        # rate = 120 / 60 = 2
        assert result["request_rate"][0] == 2.0

    def test_azure2019_cpu_from_rate_and_duration(self, tmp_path):
        """cpu_millis = request_rate * Average."""
        invocations = tmp_path / "invocations.csv"
        durations = tmp_path / "durations.csv"
        memory = tmp_path / "memory.csv"

        invocations.write_text("HashOwner,HashApp,HashFunction,Trigger,1\no1,app1,f1,http,120\n")
        durations.write_text(
            "HashOwner,HashApp,HashFunction,Average,Count,Minimum,Maximum\n"
            "o1,app1,f1,100,100,50,200\n"
        )
        memory.write_text("HashOwner,HashApp,SampleCount,AverageAllocatedMb\no1,app1,100,256\n")

        result = azure2019.load_day(1, invocations, durations, memory)
        # rate = 2, duration = 100 → cpu = 2 * 100 = 200
        assert result["cpu_millis"][0] == 200.0

    def test_azure2019_memory_computation(self, tmp_path):
        """memory_bytes = AverageAllocatedMb * 2**20."""
        invocations = tmp_path / "invocations.csv"
        durations = tmp_path / "durations.csv"
        memory = tmp_path / "memory.csv"

        invocations.write_text("HashOwner,HashApp,HashFunction,Trigger,1\no1,app1,f1,http,60\n")
        durations.write_text(
            "HashOwner,HashApp,HashFunction,Average,Count,Minimum,Maximum\n"
            "o1,app1,f1,100,100,50,200\n"
        )
        memory.write_text("HashOwner,HashApp,SampleCount,AverageAllocatedMb\no1,app1,100,256\n")

        result = azure2019.load_day(1, invocations, durations, memory)
        expected = 256 * (2**20)
        assert result["memory_bytes"][0] == expected

    def test_azure2019_memory_none_is_zero(self, tmp_path):
        """memory_bytes = 0 when memory data missing."""
        invocations = tmp_path / "invocations.csv"
        durations = tmp_path / "durations.csv"

        invocations.write_text("HashOwner,HashApp,HashFunction,Trigger,1\no1,app1,f1,http,60\n")
        durations.write_text(
            "HashOwner,HashApp,HashFunction,Average,Count,Minimum,Maximum\n"
            "o1,app1,f1,100,100,50,200\n"
        )

        result = azure2019.load_day(1, invocations, durations, None)
        assert result["memory_bytes"][0] == 0.0

    def test_azure2019_day_2_offset(self, tmp_path):
        """Day 2 offsets by 86400."""
        invocations = tmp_path / "invocations.csv"
        durations = tmp_path / "durations.csv"
        memory = tmp_path / "memory.csv"

        invocations.write_text("HashOwner,HashApp,HashFunction,Trigger,1\no1,app1,f1,http,60\n")
        durations.write_text(
            "HashOwner,HashApp,HashFunction,Average,Count,Minimum,Maximum\n"
            "o1,app1,f1,100,100,50,200\n"
        )
        memory.write_text("HashOwner,HashApp,SampleCount,AverageAllocatedMb\no1,app1,100,256\n")

        result = azure2019.load_day(2, invocations, durations, memory)
        assert result["time_s"][0] == 86400.0


class TestBorg2019:
    """Test borg2019 loader."""

    def test_borg2019_load_basic(self, tmp_path):
        """Load produces correct columns."""
        usage = tmp_path / "instance_usage_0.csv"
        usage.write_text(
            "start_time,collection_id,instance_index,average_usage_cpus,average_usage_memory\n"
            "0,coll1,0,0.5,0.4\n"
        )

        result = borg2019.load([usage])
        assert isinstance(result, pl.DataFrame)
        assert set(result.columns) >= {
            "time_s",
            "service",
            "instance",
            "cpu_millis",
            "memory_bytes",
            "request_rate",
        }

    def test_borg2019_time_conversion(self, tmp_path):
        """time_s = start_time / 1e6."""
        usage = tmp_path / "instance_usage_0.csv"
        usage.write_text(
            "start_time,collection_id,instance_index,average_usage_cpus,average_usage_memory\n"
            "1000000,coll1,0,0.5,0.4\n"
        )

        result = borg2019.load([usage])
        assert result["time_s"][0] == 1.0

    def test_borg2019_cpu_millis(self, tmp_path):
        """cpu_millis = average_usage_cpus * cpu_millis_per_unit."""
        usage = tmp_path / "instance_usage_0.csv"
        usage.write_text(
            "start_time,collection_id,instance_index,average_usage_cpus,average_usage_memory\n"
            "0,coll1,0,0.5,0.4\n"
        )

        result = borg2019.load([usage])
        # cpu = 0.5 * 1000 = 500
        assert result["cpu_millis"][0] == 500.0

    def test_borg2019_memory_millis(self, tmp_path):
        """memory_bytes = average_usage_memory * memory_bytes_per_unit."""
        usage = tmp_path / "instance_usage_0.csv"
        usage.write_text(
            "start_time,collection_id,instance_index,average_usage_cpus,average_usage_memory\n"
            "0,coll1,0,0.5,0.4\n"
        )

        result = borg2019.load([usage])
        expected = 0.4 * (2**30)
        assert result["memory_bytes"][0] == pytest.approx(expected)

    def test_borg2019_request_rate_from_cpu(self, tmp_path):
        """request_rate = cpu_millis / cpu_millis_per_request."""
        usage = tmp_path / "instance_usage_0.csv"
        usage.write_text(
            "start_time,collection_id,instance_index,average_usage_cpus,average_usage_memory\n"
            "0,coll1,0,0.5,0.4\n"
        )

        result = borg2019.load([usage])
        # cpu = 500, rate = 500 / 50 = 10
        assert result["request_rate"][0] == 10.0
