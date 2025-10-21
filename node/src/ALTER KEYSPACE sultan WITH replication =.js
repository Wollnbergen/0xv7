ALTER KEYSPACE sultan WITH replication = {
  'class': 'NetworkTopologyStrategy',
  'datacenter1': '3'
};